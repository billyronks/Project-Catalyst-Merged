// Voice Switch kdb+ Gateway
// Load balances queries across RDB and HDB instances

// Configuration
\d .gw
servers:`rdb`hdb!(`$":kdb-rdb:5012";`$":kdb:5000")
handles:`rdb`hdb!0Ni 0Ni
timeout:30000                // Query timeout in ms
maxRetries:3                 // Max connection retries

\d .

// Connect to backend servers
.gw.connect:{
    -1 "Connecting to backend servers...";
    .gw.handles[`rdb]:@[hopen;.gw.servers`rdb;{-1 "RDB connection failed: ",x; 0Ni}];
    .gw.handles[`hdb]:@[hopen;.gw.servers`hdb;{-1 "HDB connection failed: ",x; 0Ni}];
    connected:sum not null .gw.handles;
    -1 "Connected to ",string[connected]," of 2 servers";
    }

// Reconnect on failure
.gw.reconnect:{[srv]
    if[null .gw.handles srv;
        .gw.handles[srv]:@[hopen;.gw.servers srv;{0Ni}]
    ];
    }

// Execute query on specific server
.gw.exec:{[srv;q]
    .gw.reconnect srv;
    if[null .gw.handles srv; '"Server ",string[srv]," unavailable"];
    @[.gw.handles[srv];q;{'"Query failed: ",x}]
    }

// Execute query on RDB (real-time data)
.gw.rdb:{[q] .gw.exec[`rdb;q]}

// Execute query on HDB (historical data)
.gw.hdb:{[q] .gw.exec[`hdb;q]}

// Smart routing - route based on time range
.gw.query:{[q;startTime;endTime]
    today:`date$.z.d;
    // If query is entirely historical, use HDB
    if[(`date$endTime)<today;
        :.gw.hdb q
    ];
    // If query is entirely real-time, use RDB
    if[(`date$startTime)>=today;
        :.gw.rdb q
    ];
    // Otherwise, need to merge results
    rdbRes:.gw.rdb q;
    hdbRes:.gw.hdb q;
    rdbRes,hdbRes
    }

// ===========================================
// High-Level Analytics API
// ===========================================

\d .api

// Get real-time traffic summary
traffic:{[minutes]
    .gw.rdb (`.analytics.realtimeTraffic;minutes)
    }

// Get carrier statistics
carrier:{[carrierId;minutes]
    .gw.rdb (`.analytics.carrierStats;carrierId;minutes)
    }

// Get all carrier statistics
carriers:{[minutes]
    .gw.rdb (`.analytics.allCarrierStats;minutes)
    }

// Get destination analytics
destinations:{[minutes;prefixLen]
    .gw.rdb (`.analytics.destStats;minutes;prefixLen)
    }

// Get hourly pattern (needs HDB for historical data)
hourlyPattern:{[days]
    .gw.query[(`.analytics.hourlyPattern;days);.z.p-`long$days*86400000000000;.z.p]
    }

// Get account usage
accountUsage:{[accountId;days]
    .gw.query[(`.analytics.accountUsage;accountId;days);.z.p-`long$days*86400000000000;.z.p]
    }

// Get billing summary
billingSummary:{[accountId;startDate;endDate]
    .gw.query[(`.billing.accountSummary;accountId;startDate;endDate);startDate;endDate]
    }

// Get QoS metrics
qos:{[carrierId;minutes]
    .gw.rdb (`.qos.carrierQuality;carrierId;minutes)
    }

// Get fraud alerts
fraudAlerts:{[minutes]
    .gw.rdb (`.rt.recentAlerts;minutes)
    }

// Get active calls
activeCalls:{
    .gw.rdb (`.rt.activeCalls;`)
    }

// Get CPS (calls per second)
cps:{[seconds]
    .gw.rdb (`.rt.cps;seconds)
    }

// Get real-time ASR
asr:{[minutes]
    .gw.rdb (`.rt.asr;minutes)
    }

// Get real-time ACD
acd:{[minutes]
    .gw.rdb (`.rt.acd;minutes)
    }

\d .

// ===========================================
// HTTP/REST API Handler
// ===========================================

.http.handler:{[req]
    path:req`path;
    params:req`params;

    result:$[
        path~"/api/v1/kdb/health";
            `status`servers!(
                `ok;
                `rdb`hdb!(not null .gw.handles`rdb;not null .gw.handles`hdb)
            );
        path~"/api/v1/kdb/traffic";
            .api.traffic[`int$params`minutes];
        path~"/api/v1/kdb/carrier";
            .api.carrier[`$params`carrierId;`int$params`minutes];
        path~"/api/v1/kdb/carriers";
            .api.carriers[`int$params`minutes];
        path~"/api/v1/kdb/destinations";
            .api.destinations[`int$params`minutes;`int$params`prefixLen];
        path~"/api/v1/kdb/hourly";
            .api.hourlyPattern[`int$params`days];
        path~"/api/v1/kdb/qos";
            .api.qos[`$params`carrierId;`int$params`minutes];
        path~"/api/v1/kdb/fraud";
            .api.fraudAlerts[`int$params`minutes];
        path~"/api/v1/kdb/active";
            .api.activeCalls[];
        path~"/api/v1/kdb/cps";
            .api.cps[`int$params`seconds];
        path~"/api/v1/kdb/asr";
            .api.asr[`int$params`minutes];
        // Default error
        `error`message!(`not_found;"Unknown endpoint")
    ];

    `status`body!(200;.j.j result)
    }

// IPC handlers
.z.pg:{value x}
.z.ps:{value x}

// Connection handlers
.z.po:{[h] -1 "Client connected: ",string h}
.z.pc:{[h] -1 "Client disconnected: ",string h}

// Initialize connections
.gw.connect[]

-1 "Voice Switch Gateway started on port ",string system "p"
