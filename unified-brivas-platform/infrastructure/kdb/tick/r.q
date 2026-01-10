// Voice Switch RDB (Real-time Database)
// Subscribes to tickerplant and maintains in-memory real-time data

// Command line arguments
tp:.z.x 0                    // Tickerplant address

// Load schema
system "l /opt/kx/schema/cdr.q"

// Configuration
\d .rdb
hdbDir:`:/data/hdb
maxRows:10000000j           // Max rows before warning
flushInterval:60000         // Flush interval in ms (1 minute)

\d .

// Update handler - receives data from tickerplant
upd:{[t;x]
    t insert x;
    // Log high volume warning
    if[.rdb.maxRows < count value t;
        -1 "WARNING: Table ",string[t]," exceeds max rows"
    ];
    }

// End of day handler
.u.end:{[d]
    -1 "End of day: saving data for ",string d;
    // Save each table to HDB
    {[d;t]
        if[0 < count value t;
            path:`$string[.rdb.hdbDir],"/",string[d],"/",string[t],"/";
            path set .Q.en[.rdb.hdbDir] value t;
            -1 "Saved ",string[t]," to ",string path
        ]
    }[d;] each tables[];
    // Clear in-memory tables
    {delete from x} each tables[];
    -1 "End of day processing complete";
    }

// Subscribe to tickerplant
.rdb.sub:{
    -1 "Connecting to tickerplant at ",tp;
    h:@[hopen;`$":",tp;{-1 "Failed to connect: ",x; 0Ni}];
    if[null h; -1 "Retrying in 5 seconds..."; .z.ts:{.rdb.sub[]}; system "t 5000"; :()];
    system "t 0";  // Disable timer
    // Subscribe to all tables
    {h (`.tick.sub;x)} each `cdr`carrierMetrics`fraudAlert`activeCalls;
    -1 "Subscribed to tickerplant";
    }

// Real-time query functions
\d .rt

// Get current active calls
activeCalls:{
    select from activeCalls where endTime=0Np
    }

// Get calls per second (last N seconds)
cps:{[n]
    t:.z.p - `long$n * 1000000000;
    (count select from cdr where time>=t) % n
    }

// Get real-time ASR (last N minutes)
asr:{[n]
    t:.z.p - `long$n * 60000000000;
    100 * avg exec status=`ANSWERED from cdr where time>=t
    }

// Get real-time ACD (last N minutes)
acd:{[n]
    t:.z.p - `long$n * 60000000000;
    avg exec duration from cdr where time>=t, status=`ANSWERED
    }

// Get carrier real-time metrics
carrierRt:{[carrierId;minutes]
    t:.z.p - `long$minutes * 60000000000;
    select
        calls:count i,
        answered:sum status=`ANSWERED,
        asr:100*avg status=`ANSWERED,
        acd:avg duration where status=`ANSWERED,
        pdd:avg pdd,
        revenue:sum revenue,
        cost:sum cost
    from cdr
    where time>=t, carrierId=carrierId
    }

// Get destination real-time metrics
destRt:{[prefix;minutes]
    t:.z.p - `long$minutes * 60000000000;
    select
        calls:count i,
        answered:sum status=`ANSWERED,
        asr:100*avg status=`ANSWERED,
        acd:avg duration where status=`ANSWERED,
        avgRate:avg ratePerMin
    from cdr
    where time>=t, destNumber like prefix,"*"
    }

// Get recent fraud alerts
recentAlerts:{[minutes]
    t:.z.p - `long$minutes * 60000000000;
    select from fraudAlert where time>=t
    }

\d .

// IPC handlers
.z.pg:{value x}
.z.ps:{value x}

// Connection handlers
.z.po:{[h] -1 "Client connected: ",string h}
.z.pc:{[h] -1 "Client disconnected: ",string h}

// Start subscription
.rdb.sub[]

-1 "Voice Switch RDB started on port ",string system "p"
