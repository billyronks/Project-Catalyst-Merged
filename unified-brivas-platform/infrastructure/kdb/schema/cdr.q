// Voice Switch CDR Schema for kdb+
// High-performance time-series schema for Call Detail Records

// ===========================================
// CDR (Call Detail Records) Table
// ===========================================
// Primary table for storing all call records
// Optimized for time-series queries and aggregations

cdr:([]
    time:`timestamp$();              // Timestamp (nanosecond precision)
    callId:`symbol$();               // Unique call identifier
    accountId:`guid$();              // Customer account UUID
    carrierId:`guid$();              // Terminating carrier UUID
    sourceNumber:`symbol$();         // Calling party number (E.164)
    destNumber:`symbol$();           // Called party number (E.164)
    destPrefix:`symbol$();           // Destination prefix for routing
    countryCode:`symbol$();          // Destination country code
    sourceIp:`int$();                // Source IP (IPv4 as int)
    destIp:`int$();                  // Destination IP (IPv4 as int)
    startTime:`timestamp$();         // Call start time
    answerTime:`timestamp$();        // Call answer time (null if not answered)
    endTime:`timestamp$();           // Call end time
    duration:`int$();                // Total duration in seconds
    billableSecs:`int$();            // Billable seconds (after rounding)
    direction:`symbol$();            // INBOUND/OUTBOUND/TRANSIT
    status:`symbol$();               // ANSWERED/FAILED/BUSY/NOANSWER/CANCEL
    sipCode:`short$();               // SIP response code
    q850Cause:`short$();             // Q.850 cause code
    cost:`float$();                  // Calculated cost
    revenue:`float$();               // Revenue (for margin calc)
    ratePerMin:`float$();            // Rate per minute applied
    pdd:`int$();                     // Post-dial delay in ms
    mos:`float$();                   // Mean Opinion Score (1-5)
    jitter:`float$();                // Jitter in ms
    packetLoss:`float$();            // Packet loss percentage
    origCodec:`symbol$();            // Originating codec
    termCodec:`symbol$();            // Terminating codec
    userAgent:`symbol$()             // SIP User-Agent
);

// Create keyed version for lookups
cdrByCallId:([callId:`symbol$()] time:`timestamp$();accountId:`guid$();status:`symbol$())

// ===========================================
// Carrier Metrics Table (Real-time)
// ===========================================
// Rolling metrics per carrier for routing decisions

carrierMetrics:([]
    time:`timestamp$();              // Metric timestamp
    carrierId:`guid$();              // Carrier UUID
    carrierName:`symbol$();          // Carrier name for display
    totalCalls:`long$();             // Total calls in window
    answeredCalls:`long$();          // Answered calls
    failedCalls:`long$();            // Failed calls
    totalMinutes:`float$();          // Total minutes
    asr:`float$();                   // Answer Seizure Ratio (0-1)
    acd:`float$();                   // Average Call Duration (seconds)
    pdd:`float$();                   // Average Post-Dial Delay (ms)
    activeCalls:`int$();             // Current active calls
    cps:`float$();                   // Calls per second
    cost:`float$();                  // Total cost
    revenue:`float$();               // Total revenue
    margin:`float$()                 // Margin (revenue - cost)
);

// ===========================================
// Destination Analytics Table
// ===========================================
// Analytics by destination prefix

destAnalytics:([]
    time:`timestamp$();              // Aggregation timestamp
    destPrefix:`symbol$();           // Destination prefix
    countryCode:`symbol$();          // Country code
    countryName:`symbol$();          // Country name
    totalCalls:`long$();             // Total calls
    answeredCalls:`long$();          // Answered calls
    totalMinutes:`float$();          // Total minutes
    asr:`float$();                   // ASR for this destination
    acd:`float$();                   // ACD for this destination
    avgRate:`float$();               // Average rate
    totalCost:`float$();             // Total cost
    totalRevenue:`float$()           // Total revenue
);

// ===========================================
// Account Balance Events (Real-time)
// ===========================================
// Streaming balance updates for prepaid accounts

balanceEvent:([]
    time:`timestamp$();              // Event timestamp
    accountId:`guid$();              // Account UUID
    eventType:`symbol$();            // CHARGE/TOPUP/RESERVE/RELEASE
    amount:`float$();                // Transaction amount
    balanceBefore:`float$();         // Balance before event
    balanceAfter:`float$();          // Balance after event
    callId:`symbol$();               // Related call ID (if applicable)
    description:`symbol$()           // Event description
);

// ===========================================
// Fraud Alerts Table
// ===========================================
// Real-time fraud detection events

fraudAlert:([]
    time:`timestamp$();              // Alert timestamp
    alertId:`guid$();                // Alert UUID
    accountId:`guid$();              // Affected account
    ruleId:`guid$();                 // Triggering rule
    ruleType:`symbol$();             // VELOCITY/DESTINATION/PATTERN/GEO
    severity:`symbol$();             // LOW/MEDIUM/HIGH/CRITICAL
    sourceIp:`int$();                // Source IP
    destNumber:`symbol$();           // Destination number
    callCount:`int$();               // Number of calls triggering
    description:`symbol$();          // Alert description
    status:`symbol$()                // NEW/INVESTIGATING/RESOLVED/DISMISSED
);

// ===========================================
// Active Calls Table (Real-time state)
// ===========================================
// Current active calls for real-time monitoring

activeCalls:([]
    callId:`symbol$();               // Unique call identifier
    accountId:`guid$();              // Account UUID
    carrierId:`guid$();              // Carrier UUID
    sourceNumber:`symbol$();         // Source number
    destNumber:`symbol$();           // Destination number
    startTime:`timestamp$();         // Call start time
    duration:`int$();                // Current duration
    direction:`symbol$();            // INBOUND/OUTBOUND
    ratePerMin:`float$();            // Rate being applied
    estimatedCost:`float$()          // Estimated current cost
);

// ===========================================
// Billing Aggregations Table
// ===========================================
// Pre-aggregated billing data for invoicing

billingAgg:([]
    date:`date$();                   // Billing date
    accountId:`guid$();              // Account UUID
    carrierId:`guid$();              // Carrier UUID
    destPrefix:`symbol$();           // Destination prefix
    totalCalls:`long$();             // Total calls
    answeredCalls:`long$();          // Answered calls
    totalMinutes:`float$();          // Total billable minutes
    totalCost:`float$();             // Total cost
    totalRevenue:`float$();          // Total revenue
    avgRate:`float$()                // Average rate applied
);

// ===========================================
// Network Quality Metrics
// ===========================================
// QoS metrics for network monitoring

qosMetrics:([]
    time:`timestamp$();              // Metric timestamp
    carrierId:`guid$();              // Carrier UUID
    avgMos:`float$();                // Average MOS
    avgJitter:`float$();             // Average jitter (ms)
    avgPacketLoss:`float$();         // Average packet loss %
    avgPdd:`float$();                // Average PDD (ms)
    p95Pdd:`float$();                // 95th percentile PDD
    p99Pdd:`float$();                // 99th percentile PDD
    callsWithIssues:`long$();        // Calls with quality issues
    totalCalls:`long$()              // Total calls measured
);

// Create indices for common queries
`time xasc `cdr;
`carrierId`time xasc `carrierMetrics;
`destPrefix`time xasc `destAnalytics;
`accountId`time xasc `balanceEvent;
`time xasc `fraudAlert;

// Log schema loaded
-1 "CDR schema loaded successfully";
