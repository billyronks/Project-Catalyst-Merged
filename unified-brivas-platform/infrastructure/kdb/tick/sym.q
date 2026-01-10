// Symbol file for tickerplant
// Defines the schema for streaming tables

// CDR table schema for tick
cdr:([]
    time:`timestamp$();
    callId:`guid$();
    accountId:`guid$();
    carrierId:`guid$();
    sourceNumber:`symbol$();
    destNumber:`symbol$();
    sourceIp:`int$();
    destIp:`int$();
    duration:`int$();
    billableSecs:`int$();
    status:`symbol$();
    sipCode:`int$();
    pdd:`float$();
    ratePerMin:`float$();
    cost:`float$();
    revenue:`float$();
    codec:`symbol$();
    mos:`float$();
    jitter:`float$();
    packetLoss:`float$()
    )

// Carrier metrics schema for tick
carrierMetrics:([]
    time:`timestamp$();
    carrierId:`guid$();
    carrierName:`symbol$();
    totalCalls:`long$();
    answeredCalls:`long$();
    failedCalls:`long$();
    totalMinutes:`float$();
    asr:`float$();
    acd:`float$();
    pdd:`float$();
    activeCalls:`int$();
    cps:`float$();
    cost:`float$();
    revenue:`float$();
    margin:`float$()
    )

// Fraud alert schema for tick
fraudAlert:([]
    time:`timestamp$();
    alertId:`guid$();
    accountId:`guid$();
    ruleId:`guid$();
    ruleType:`symbol$();
    severity:`symbol$();
    sourceIp:`int$();
    destNumber:`symbol$();
    callCount:`int$();
    description:`symbol$();
    status:`symbol$()
    )

// Active calls schema for tick
activeCalls:([]
    time:`timestamp$();
    callId:`guid$();
    accountId:`guid$();
    carrierId:`guid$();
    sourceNumber:`symbol$();
    destNumber:`symbol$();
    startTime:`timestamp$();
    endTime:`timestamp$();
    status:`symbol$()
    )
