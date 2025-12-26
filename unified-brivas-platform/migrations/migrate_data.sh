#!/bin/bash
# Data Migration Script: MySQL/PostgreSQL/MongoDB -> LumaDB
# This script migrates data from existing databases to unified LumaDB

set -e

# Configuration
LUMADB_HOST="${LUMADB_HOST:-localhost}"
LUMADB_PORT="${LUMADB_PORT:-5432}"
LUMADB_USER="${LUMADB_USER:-brivas}"
LUMADB_DB="${LUMADB_DB:-brivas}"

MYSQL_HOST="${MYSQL_HOST:-localhost}"
MYSQL_PORT="${MYSQL_PORT:-3306}"
MYSQL_USER="${MYSQL_USER:-root}"
MYSQL_DB="${MYSQL_DB:-brivas}"

MONGO_HOST="${MONGO_HOST:-localhost}"
MONGO_PORT="${MONGO_PORT:-27017}"
MONGO_DB="${MONGO_DB:-brivas}"

echo "============================================"
echo "Unified Brivas Platform - Data Migration"
echo "============================================"

# Step 1: Apply LumaDB schema
echo ""
echo "[Step 1/5] Applying LumaDB schema..."
PGPASSWORD="${LUMADB_PASSWORD}" psql -h "$LUMADB_HOST" -p "$LUMADB_PORT" -U "$LUMADB_USER" -d "$LUMADB_DB" \
    -f ./migrations/lumadb/001_initial_schema.sql
echo "  ✓ Schema applied"

# Step 2: Migrate accounts from MySQL
echo ""
echo "[Step 2/5] Migrating accounts from MySQL..."
mysql -h "$MYSQL_HOST" -P "$MYSQL_PORT" -u "$MYSQL_USER" -p"${MYSQL_PASSWORD}" "$MYSQL_DB" \
    -e "SELECT id, email, first_name, last_name, phone_number, test_secret_key, live_secret_key, balance, is_blacklist, rates, reg_time FROM accounts" \
    --batch --skip-column-names | \
while IFS=$'\t' read -r id email first_name last_name phone test_key live_key balance blacklist rates reg_time; do
    PGPASSWORD="${LUMADB_PASSWORD}" psql -h "$LUMADB_HOST" -p "$LUMADB_PORT" -U "$LUMADB_USER" -d "$LUMADB_DB" \
        -c "INSERT INTO accounts (id, email, first_name, last_name, phone_number, test_secret_key, live_secret_key, balance, is_blacklist, rates, reg_time) 
            VALUES ('$id', '$email', '$first_name', '$last_name', '$phone', '$test_key', '$live_key', $balance, $blacklist, '$rates', '$reg_time')
            ON CONFLICT (id) DO NOTHING;"
done
echo "  ✓ Accounts migrated"

# Step 3: Migrate SMS history from MySQL
echo ""
echo "[Step 3/5] Migrating SMS history from MySQL..."
mysql -h "$MYSQL_HOST" -P "$MYSQL_PORT" -u "$MYSQL_USER" -p"${MYSQL_PASSWORD}" "$MYSQL_DB" \
    -e "SELECT accountId, sid, rid, \`from\`, \`to\`, status, type, msg, ratePerSMS, date, time FROM sms_history ORDER BY id DESC LIMIT 1000000" \
    --batch --skip-column-names | \
while IFS=$'\t' read -r account_id sid rid sender recipient status type message rate date time; do
    PGPASSWORD="${LUMADB_PASSWORD}" psql -h "$LUMADB_HOST" -p "$LUMADB_PORT" -U "$LUMADB_USER" -d "$LUMADB_DB" \
        -c "INSERT INTO sms_history (account_id, sid, rid, sender, recipient, status, type, message, rate_per_sms, sent_date, sent_time) 
            VALUES ('$account_id', '$sid', '$rid', '$sender', '$recipient', '$status', '$type', '$message', $rate, '$date', '$time')
            ON CONFLICT DO NOTHING;" 2>/dev/null || true
done
echo "  ✓ SMS history migrated (last 1M records)"

# Step 4: Migrate sender IDs from MySQL
echo ""
echo "[Step 4/5] Migrating sender IDs from MySQL..."
mysql -h "$MYSQL_HOST" -P "$MYSQL_PORT" -u "$MYSQL_USER" -p"${MYSQL_PASSWORD}" "$MYSQL_DB" \
    -e "SELECT accountId, sender, status, type, approved, is_public, is_general FROM senderIds" \
    --batch --skip-column-names | \
while IFS=$'\t' read -r account_id sender status type approved is_public is_general; do
    PGPASSWORD="${LUMADB_PASSWORD}" psql -h "$LUMADB_HOST" -p "$LUMADB_PORT" -U "$LUMADB_USER" -d "$LUMADB_DB" \
        -c "INSERT INTO sender_ids (account_id, sender, status, type, approved, is_public, is_general) 
            VALUES ('$account_id', '$sender', '$status', '$type', $approved, $is_public, $is_general)
            ON CONFLICT (sender, type) DO NOTHING;"
done
echo "  ✓ Sender IDs migrated"

# Step 5: Migrate contacts from MongoDB
echo ""
echo "[Step 5/5] Migrating contacts from MongoDB..."
mongosh --host "$MONGO_HOST" --port "$MONGO_PORT" "$MONGO_DB" --eval "
    db.contacts.find().forEach(function(doc) {
        print(doc.accountId + '\t' + doc.uid + '\t' + doc.name + '\t' + JSON.stringify(doc.numbers));
    })
" | while IFS=$'\t' read -r account_id uid name numbers; do
    PGPASSWORD="${LUMADB_PASSWORD}" psql -h "$LUMADB_HOST" -p "$LUMADB_PORT" -U "$LUMADB_USER" -d "$LUMADB_DB" \
        -c "INSERT INTO contacts (account_id, uid, name, numbers) 
            VALUES ('$account_id', '$uid', '$name', '$numbers'::jsonb)
            ON CONFLICT (uid) DO NOTHING;"
done
echo "  ✓ Contacts migrated"

echo ""
echo "============================================"
echo "Migration Complete!"
echo "============================================"
echo ""
echo "Summary:"
PGPASSWORD="${LUMADB_PASSWORD}" psql -h "$LUMADB_HOST" -p "$LUMADB_PORT" -U "$LUMADB_USER" -d "$LUMADB_DB" -c "
SELECT 'accounts' as table_name, COUNT(*) as record_count FROM accounts
UNION ALL
SELECT 'sms_history', COUNT(*) FROM sms_history
UNION ALL
SELECT 'sender_ids', COUNT(*) FROM sender_ids
UNION ALL
SELECT 'contacts', COUNT(*) FROM contacts;
"
