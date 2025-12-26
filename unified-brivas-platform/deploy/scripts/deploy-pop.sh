#!/bin/bash
# deploy-pop.sh - Automated PoP Deployment Script (5 Tier-1 PoPs)
#
# Deploys a complete Point of Presence with:
# - K3s/K8s cluster
# - Cilium eBPF load balancing
# - LumaDB
# - All 27 microservices
# - Voice infrastructure (OpenSIPS, FreeSWITCH, Kamailio, rtpengine)

set -euo pipefail

# ============================================================================
# CONFIGURATION
# ============================================================================
POP_ID="${1:?Usage: $0 <pop_id> [--dry-run]}"
DRY_RUN="${2:-}"

# Determine PoP configuration
case $POP_ID in
  lagos-ng-1)
    SUBNET="102.217.214.0/25"
    VIP_BASE="102.217.214"
    STIR_SHAKEN="false"
    REGION="africa"
    ;;
  london-uk-1)
    SUBNET="185.230.50.0/25"
    VIP_BASE="185.230.50"
    STIR_SHAKEN="false"
    REGION="europe"
    ;;
  ashburn-us-1)
    SUBNET="198.51.100.0/25"
    VIP_BASE="198.51.100"
    STIR_SHAKEN="true"  # FCC mandate
    REGION="americas-north"
    ;;
  saopaulo-br-1)
    SUBNET="177.85.40.0/25"
    VIP_BASE="177.85.40"
    STIR_SHAKEN="false"
    REGION="americas-south"
    ;;
  singapore-sg-1)
    SUBNET="103.200.60.0/25"
    VIP_BASE="103.200.60"
    STIR_SHAKEN="false"
    REGION="apac"
    ;;
  *)
    echo "ERROR: Unknown PoP: $POP_ID"
    echo "Valid PoPs: lagos-ng-1, london-uk-1, ashburn-us-1, saopaulo-br-1, singapore-sg-1"
    exit 1
    ;;
esac

KUBE_VIP="${VIP_BASE}.2"
API_VIP="${VIP_BASE}.3"
PRIMARY_INTERFACE="${PRIMARY_INTERFACE:-eth0}"

echo "==================================================================="
echo "  BRIVAS PoP Deployment: $POP_ID"
echo "==================================================================="
echo "Region:          $REGION"
echo "Subnet:          $SUBNET"
echo "VIP Base:        $VIP_BASE"
echo "Kube API VIP:    $KUBE_VIP"
echo "API Gateway VIP: $API_VIP"
echo "STIR/SHAKEN:     $STIR_SHAKEN"
echo "Interface:       $PRIMARY_INTERFACE"
echo "==================================================================="

if [[ "$DRY_RUN" == "--dry-run" ]]; then
  echo "[DRY RUN MODE - No changes will be made]"
fi

run_cmd() {
  if [[ "$DRY_RUN" == "--dry-run" ]]; then
    echo "[DRY RUN] $*"
  else
    echo ">>> $*"
    eval "$@"
  fi
}

# ============================================================================
# STEP 1: KUBERNETES INSTALLATION
# ============================================================================
echo ""
echo ">>> Step 1: Installing Kubernetes..."

run_cmd "curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC='server \
    --cluster-init \
    --disable=traefik \
    --disable=servicelb \
    --flannel-backend=none \
    --disable-network-policy \
    --tls-san=$KUBE_VIP' sh -"

echo "Kubernetes installation complete."

# ============================================================================
# STEP 2: CILIUM eBPF INSTALLATION
# ============================================================================
echo ""
echo ">>> Step 2: Installing Cilium eBPF Load Balancer..."

run_cmd "helm repo add cilium https://helm.cilium.io/"
run_cmd "helm repo update"

# Generate random hash seed for this PoP
HASH_SEED=$(openssl rand -hex 16)

cat > /tmp/cilium-values-${POP_ID}.yaml <<EOF
kubeProxyReplacement: strict
k8sServiceHost: $KUBE_VIP
k8sServicePort: 6443

bpf:
  masquerade: true
  clockProbe: true
  preallocateMaps: true
  lbMapMax: 512000
  ctTcpMax: 2097152

loadBalancer:
  mode: dsr
  dsrDispatch: opt
  acceleration: native
  algorithm: maglev
  maglev:
    tableSize: 65521
    hashSeed: "$HASH_SEED"

l2announcements:
  enabled: true

socketLB:
  enabled: true

bandwidthManager:
  enabled: true
  bbr: true

hubble:
  enabled: true
  relay:
    enabled: true

clustermesh:
  useAPIServer: true
EOF

run_cmd "helm upgrade --install cilium cilium/cilium -n kube-system -f /tmp/cilium-values-${POP_ID}.yaml"

echo "Cilium installation complete."

# ============================================================================
# STEP 3: VIP POOL CONFIGURATION
# ============================================================================
echo ""
echo ">>> Step 3: Configuring VIP Pool..."

cat > /tmp/vip-pool-${POP_ID}.yaml <<EOF
apiVersion: cilium.io/v2alpha1
kind: CiliumLoadBalancerIPPool
metadata:
  name: ${POP_ID}-vip-pool
spec:
  cidrs:
$(for i in $(seq 3 62); do echo "    - cidr: \"${VIP_BASE}.$i/32\""; done)
  serviceSelector:
    matchLabels:
      io.cilium/lb-ipam-layer2: "true"
---
apiVersion: cilium.io/v2alpha1
kind: CiliumL2AnnouncementPolicy
metadata:
  name: ${POP_ID}-l2-policy
spec:
  serviceSelector:
    matchExpressions:
      - key: io.cilium/l2-announcement
        operator: Exists
  interfaces:
    - $PRIMARY_INTERFACE
  externalIPs: true
  loadBalancerIPs: true
EOF

run_cmd "kubectl apply -f /tmp/vip-pool-${POP_ID}.yaml"

echo "VIP Pool configured."

# ============================================================================
# STEP 4: LUMADB DEPLOYMENT
# ============================================================================
echo ""
echo ">>> Step 4: Deploying LumaDB..."

run_cmd "helm upgrade --install lumadb ./charts/lumadb \
    --namespace brivas --create-namespace \
    --set pop.id=$POP_ID \
    --set federation.enabled=true \
    --set replication.mode=async \
    --set replication.factor=5"

echo "LumaDB deployed."

# ============================================================================
# STEP 5: VOICE INFRASTRUCTURE DEPLOYMENT
# ============================================================================
echo ""
echo ">>> Step 5: Deploying Voice Infrastructure..."

run_cmd "helm upgrade --install voice-stack ./charts/voice-stack \
    --namespace brivas \
    --set pop.id=$POP_ID \
    --set kamailio.replicas=3 \
    --set opensips.replicas=5 \
    --set freeswitch.replicas=5 \
    --set rtpengine.replicas=3 \
    --set stirShaken.enabled=$STIR_SHAKEN"

echo "Voice infrastructure deployed."

# ============================================================================
# STEP 6: MICROSERVICES DEPLOYMENT
# ============================================================================
echo ""
echo ">>> Step 6: Deploying 27 Microservices..."

cat > /tmp/brivas-values-${POP_ID}.yaml <<EOF
pop:
  id: $POP_ID
  region: $REGION
  subnet: $SUBNET
  vipBase: $VIP_BASE
  kubeApiVip: $KUBE_VIP
  primaryInterface: $PRIMARY_INTERFACE
  stirShaken: $STIR_SHAKEN

global:
  imageTag: latest
  lumadb:
    host: lumadb.brivas.svc.cluster.local
    port: 5432
    database: brivas

voice:
  stirShaken:
    enabled: $STIR_SHAKEN
EOF

run_cmd "helm upgrade --install brivas-platform ./charts/brivas-platform \
    --namespace brivas \
    -f /tmp/brivas-values-${POP_ID}.yaml"

echo "Microservices deployed."

# ============================================================================
# STEP 7: CLUSTER MESH SETUP
# ============================================================================
echo ""
echo ">>> Step 7: Configuring Cluster Mesh..."

run_cmd "cilium clustermesh enable"

echo "Cluster mesh configured."

# ============================================================================
# STEP 8: VERIFICATION
# ============================================================================
echo ""
echo ">>> Step 8: Verification Commands..."
echo ""
echo "Run these commands to verify deployment:"
echo ""
echo "  kubectl -n brivas get pods"
echo "  kubectl -n brivas get svc"
echo "  cilium status"
echo "  cilium service list"
echo "  cilium connectivity test"
echo ""

# ============================================================================
# SUMMARY
# ============================================================================
echo "==================================================================="
echo "  PoP $POP_ID Deployment Complete!"
echo "==================================================================="
echo ""
echo "VIP Assignments (27 services):"
echo "  Kube API:        $KUBE_VIP:6443"
echo "  API Gateway:     ${VIP_BASE}.3:443"
echo "  SMSC:            ${VIP_BASE}.4:2775"
echo "  USSD Gateway:    ${VIP_BASE}.6:8080"
echo "  Messaging Hub:   ${VIP_BASE}.8:8080"
echo "  Voice/IVR:       ${VIP_BASE}.27:5060"
echo ""
echo "STIR/SHAKEN:       $STIR_SHAKEN"
echo ""
echo "Next Steps:"
echo "  1. Configure DNS to point to API Gateway VIP"
echo "  2. Connect to cluster mesh with peer PoPs:"
echo "     cilium clustermesh connect --destination-context=<peer-context>"
echo "  3. Run connectivity tests"
echo "  4. Monitor with: kubectl -n brivas logs -l app=pop-controller"
echo ""
