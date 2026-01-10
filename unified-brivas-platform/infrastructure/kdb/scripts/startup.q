// Voice Switch kdb+ Startup Script
// Initializes the analytics engine with schemas, functions, and APIs

system "l /opt/kx/schema/cdr.q"

// ===========================================
// Configuration
// ===========================================
.cfg.dataDir:"/data";
.cfg.hdbDir:"/data/hdb";
.cfg.logDir:"/data/logs";
.cfg.retentionDays:90;

// ===========================================
// Utility Functions
// ===========================================

// Convert IP integer to string
.util.ipToStr:{[ip] "." sv string `int$0x000000ff land/:ip div\: 16777216 65536 256 1}

// Parse E.164 number to get country code
.util.getCountryCode:{[num]
    n:$[num like "+*";1_num;num];
    // Common country code prefixes
    $[n like "1*";"1";          // USA/Canada
      n like "44*";"44";        // UK
      n like "49*";"49";        // Germany
      n like "33*";"33";        // France
      n like "234*";"234";      // Nigeria
      n like "27*";"27";        // South Africa
      n like "91*";"91";        // India
      n like "86*";"86";        // China
      n like "81*";"81";        // Japan
      ""]
}

// ===========================================
// CDR Ingestion Functions
// ===========================================

// Insert single CDR record
.cdr.insert:{[rec]
    `cdr insert rec;
    // Update carrier metrics
    .metrics.updateCarrier rec;
    // Check fraud rules
    .fraud.checkCall rec;
    rec`callId
}

// Bulk insert CDRs (high performance)
.cdr.bulkInsert:{[recs]
    `cdr insert recs;
    // Batch update metrics
    .metrics.batchUpdate recs;
    count recs
}

// ===========================================
// Real-Time Analytics Functions
// ===========================================

// Get carrier statistics for last N minutes
.analytics.carrierStats:{[carrierId;minutes]
    startTime:.z.p - `long$minutes * 60000000000;
    select
        totalCalls:count i,
        answeredCalls:sum status=`ANSWERED,
        failedCalls:sum status<>`ANSWERED,
        totalMinutes:sum billableSecs%60.0,
        asr:avg status=`ANSWERED,
        acd:avg duration where status=`ANSWERED,
        avgPdd:avg pdd,
        totalCost:sum cost,
        totalRevenue:sum revenue,
        margin:sum revenue-cost
    from cdr
    where time>=startTime, carrierId=carrierId
}

// Get all carrier performance summary
.analytics.allCarrierStats:{[minutes]
    startTime:.z.p - `long$minutes * 60000000000;
    select
        totalCalls:count i,
        answeredCalls:sum status=`ANSWERED,
        failedCalls:sum status<>`ANSWERED,
        totalMinutes:sum billableSecs%60.0,
        asr:avg status=`ANSWERED,
        acd:avg duration where status=`ANSWERED,
        avgPdd:avg pdd,
        totalCost:sum cost,
        totalRevenue:sum revenue
    by carrierId
    from cdr
    where time>=startTime
}

// Get real-time traffic statistics
.analytics.realtimeTraffic:{[minutes]
    startTime:.z.p - `long$minutes * 60000000000;
    select
        totalCalls:count i,
        answeredCalls:sum status=`ANSWERED,
        totalMinutes:sum billableSecs%60.0,
        asr:100*avg status=`ANSWERED,
        acd:avg duration where status=`ANSWERED,
        cps:(count i)%(minutes*60.0),
        totalCost:sum cost,
        totalRevenue:sum revenue,
        margin:sum revenue-cost
    from cdr
    where time>=startTime
}

// Get destination analytics
.analytics.destStats:{[minutes;prefixLen]
    startTime:.z.p - `long$minutes * 60000000000;
    select
        totalCalls:count i,
        answeredCalls:sum status=`ANSWERED,
        totalMinutes:sum billableSecs%60.0,
        asr:100*avg status=`ANSWERED,
        acd:avg duration where status=`ANSWERED,
        avgRate:avg ratePerMin,
        totalCost:sum cost
    by destPrefix:prefixLen$'string destNumber
    from cdr
    where time>=startTime
}

// Get hourly traffic pattern
.analytics.hourlyPattern:{[days]
    startTime:.z.p - `long$days * 86400000000000;
    select
        totalCalls:count i,
        answeredCalls:sum status=`ANSWERED,
        asr:100*avg status=`ANSWERED,
        totalMinutes:sum billableSecs%60.0
    by hour:`hh$time
    from cdr
    where time>=startTime
}

// Get account usage summary
.analytics.accountUsage:{[accountId;days]
    startTime:.z.p - `long$days * 86400000000000;
    select
        totalCalls:count i,
        answeredCalls:sum status=`ANSWERED,
        totalMinutes:sum billableSecs%60.0,
        totalCost:sum cost,
        avgCallDuration:avg duration where status=`ANSWERED,
        topDestinations:5#desc count i by destPrefix
    from cdr
    where time>=startTime, accountId=accountId
}

// ===========================================
// Fraud Detection Functions
// ===========================================

// Check call against fraud rules
.fraud.checkCall:{[rec]
    // Velocity check - too many calls in short time
    if[.fraud.velocityCheck[rec`accountId;rec`sourceIp];
        .fraud.raiseAlert[rec;`VELOCITY;`HIGH;"High call velocity detected"]
    ];
    // Destination check - premium/high-risk destinations
    if[.fraud.destCheck[rec`destNumber];
        .fraud.raiseAlert[rec;`DESTINATION;`MEDIUM;"Call to high-risk destination"]
    ];
    // Pattern check - sequential dialing
    if[.fraud.patternCheck[rec`sourceNumber;rec`destNumber];
        .fraud.raiseAlert[rec;`PATTERN;`MEDIUM;"Sequential dialing pattern detected"]
    ];
}

// Velocity fraud check
.fraud.velocityCheck:{[accountId;sourceIp]
    startTime:.z.p - 60000000000; // Last 60 seconds
    callCount:exec count i from cdr where time>=startTime, accountId=accountId;
    callCount > 50 // More than 50 calls/minute is suspicious
}

// High-risk destination check
.fraud.destCheck:{[destNum]
    // Check against high-risk prefixes (premium, satellite, etc.)
    riskPrefixes:`$("900";"976";"809";"284";"473");
    prefix:3$string destNum;
    (`$prefix) in riskPrefixes
}

// Sequential dialing pattern check
.fraud.patternCheck:{[sourceNum;destNum]
    // Check if destination numbers are sequential
    startTime:.z.p - 300000000000; // Last 5 minutes
    recentDests:exec destNumber from cdr where time>=startTime, sourceNumber=sourceNum;
    if[3>count recentDests;:0b];
    // Check if destinations are sequential
    diffs:1_deltas `long$recentDests;
    all diffs within 1 10
}

// Raise fraud alert
.fraud.raiseAlert:{[rec;ruleType;severity;desc]
    alert:(
        .z.p;                           // time
        -1 0Ng;                         // alertId (generate UUID)
        rec`accountId;                  // accountId
        0Ng;                            // ruleId
        ruleType;                       // ruleType
        severity;                       // severity
        rec`sourceIp;                   // sourceIp
        rec`destNumber;                 // destNumber
        1i;                             // callCount
        `$desc;                         // description
        `NEW                            // status
    );
    `fraudAlert insert alert;
    -1 "FRAUD ALERT: ",desc," Account: ",string rec`accountId;
}

// ===========================================
// Carrier Metrics Functions
// ===========================================

// Update carrier metrics after each call
.metrics.updateCarrier:{[rec]
    cid:rec`carrierId;
    // Get current metrics or initialize
    metrics:$[cid in exec carrierId from carrierMetrics;
        exec first each (time;totalCalls;answeredCalls;failedCalls;totalMinutes;asr;acd;pdd;cost;revenue)
            from carrierMetrics where carrierId=cid;
        (.z.p;0j;0j;0j;0f;0f;0f;0f;0f;0f)
    ];
    // Update metrics
    newMetrics:(
        .z.p;                                           // time
        cid;                                            // carrierId
        `$"";                                           // carrierName
        metrics[1]+1;                                   // totalCalls
        metrics[2]+rec[`status]=`ANSWERED;              // answeredCalls
        metrics[3]+rec[`status]<>`ANSWERED;             // failedCalls
        metrics[4]+rec[`billableSecs]%60.0;             // totalMinutes
        0f;                                             // asr (recalculated)
        metrics[6];                                     // acd
        metrics[7];                                     // pdd
        0i;                                             // activeCalls
        0f;                                             // cps
        metrics[8]+rec[`cost];                          // cost
        metrics[9]+rec[`revenue];                       // revenue
        0f                                              // margin
    );
    // Recalculate ASR
    newMetrics[6]:newMetrics[4]%newMetrics[3];
    newMetrics[14]:newMetrics[13]-newMetrics[12]; // margin
    `carrierMetrics insert newMetrics;
}

// Batch update metrics
.metrics.batchUpdate:{[recs]
    carriers:distinct recs`carrierId;
    {.metrics.updateCarrier x} each recs;
}

// ===========================================
// Billing Functions
// ===========================================

// Get billing summary for account
.billing.accountSummary:{[accountId;startDate;endDate]
    select
        totalCalls:count i,
        answeredCalls:sum status=`ANSWERED,
        totalMinutes:sum billableSecs%60.0,
        totalCost:sum cost,
        totalRevenue:sum revenue,
        margin:sum revenue-cost
    by `date$time
    from cdr
    where time>=startDate, time<endDate, accountId=accountId
}

// Get detailed billing by destination
.billing.destBreakdown:{[accountId;startDate;endDate]
    select
        totalCalls:count i,
        totalMinutes:sum billableSecs%60.0,
        totalCost:sum cost,
        avgRate:avg ratePerMin
    by destPrefix:3$'string destNumber
    from cdr
    where time>=startDate, time<endDate, accountId=accountId, status=`ANSWERED
}

// ===========================================
// QoS Monitoring Functions
// ===========================================

// Get QoS metrics for carrier
.qos.carrierQuality:{[carrierId;minutes]
    startTime:.z.p - `long$minutes * 60000000000;
    select
        avgMos:avg mos,
        avgJitter:avg jitter,
        avgPacketLoss:avg packetLoss,
        avgPdd:avg pdd,
        p95Pdd:pdd[`long$0.95*count pdd],
        p99Pdd:pdd[`long$0.99*count pdd],
        callsWithIssues:sum (mos<3.5) or (jitter>50) or (packetLoss>2),
        totalCalls:count i
    from cdr
    where time>=startTime, carrierId=carrierId, status=`ANSWERED
}

// ===========================================
// HTTP/REST API Handlers
// ===========================================

// Simple HTTP handler for REST API
.http.handler:{[req]
    path:req[`path];
    params:req[`params];

    $[path~"/health";
        .http.json (enlist`status)!enlist`ok;
      path~"/api/v1/analytics/traffic";
        .http.json .analytics.realtimeTraffic[`int$params`minutes];
      path~"/api/v1/analytics/carriers";
        .http.json .analytics.allCarrierStats[`int$params`minutes];
      path~"/api/v1/analytics/destinations";
        .http.json .analytics.destStats[`int$params`minutes;`int$params`prefixLen];
      .http.json (enlist`error)!enlist"Unknown endpoint"
    ]
}

// JSON response helper
.http.json:{[data]
    `status`body!(200;.j.j data)
}

// ===========================================
// IPC Message Handlers
// ===========================================

// Handle incoming messages
.z.pg:{[x] value x}       // Synchronous handler
.z.ps:{[x] value x}       // Asynchronous handler

// Connection handler
.z.po:{[h] -1 "Connection opened: ",string h}
.z.pc:{[h] -1 "Connection closed: ",string h}

// ===========================================
// Startup Initialization
// ===========================================

// Create data directories
system "mkdir -p ",.cfg.dataDir;
system "mkdir -p ",.cfg.hdbDir;
system "mkdir -p ",.cfg.logDir;

// Start HTTP server on port 5001
// system "p 5001"

-1 "Voice Switch kdb+ Analytics Engine started";
-1 "IPC Port: ",string system "p";
-1 "Tables: cdr, carrierMetrics, destAnalytics, balanceEvent, fraudAlert, activeCalls";
-1 "Ready for connections...";
