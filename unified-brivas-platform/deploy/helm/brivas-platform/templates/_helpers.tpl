{{- define "brivas.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "brivas.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{- define "brivas.labels" -}}
helm.sh/chart: {{ include "brivas.chart" . }}
{{ include "brivas.selectorLabels" . }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{- define "brivas.selectorLabels" -}}
app.kubernetes.io/name: {{ include "brivas.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{- define "brivas.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{- define "brivas.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "brivas.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}
