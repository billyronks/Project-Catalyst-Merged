// Voice Switch Tickerplant
// Handles real-time streaming of CDR and metrics data

// Configuration
\d .tick
src:`:sym
dst:`:hdb
log:`:logdir

// Schema definitions (loaded from schema file)
system "l /opt/kx/schema/cdr.q"

// Tickerplant state
i:0j                    // Message counter
L:()                    // Log handle
w:()                    // Subscriber handles
d:()                    // Subscriber tables

// Initialize tickerplant
init:{
    .tick.L:hopen `$":",.tick.log;
    -1 "Tickerplant initialized, logging to ",string .tick.log;
    }

// Publish data to subscribers
pub:{[t;x]
    // Write to log
    .tick.L enlist (`upd;t;x);
    // Publish to subscribers
    {[t;x;h] neg[h] (`upd;t;x)} [t;x;] each .tick.w[t];
    .tick.i+:count x;
    }

// Handle subscription requests
sub:{[t]
    if[not t in key .tick.d;
        .tick.d[t]:();
        .tick.w[t]:()
    ];
    .tick.w[t],:enlist .z.w;
    (t;value t)
    }

// End of day processing
eod:{[d]
    // Close current log
    if[not null .tick.L; hclose .tick.L];
    // Archive log file
    logfile:`$string[.tick.log],"_",ssr[string d;".";""];
    system "mv ",string[.tick.log]," ",string logfile;
    // Reinitialize
    .tick.L:hopen .tick.log;
    // Notify subscribers
    {[h] neg[h] (`.u.end;d)} each distinct raze value .tick.w;
    -1 "End of day processing complete for ",string d;
    }

\d .

// Message handlers
upd:{[t;x] .tick.pub[t;x]}

// IPC handlers
.z.pg:{value x}
.z.ps:{value x}

// Connection handlers
.z.po:{[h] -1 "Subscriber connected: ",string h}
.z.pc:{[h]
    // Remove disconnected subscriber
    .tick.w:{[w;h] key[w]!{x except h}'[value w]}[.tick.w;h];
    -1 "Subscriber disconnected: ",string h
    }

// Initialize on startup
.tick.init[]
-1 "Voice Switch Tickerplant started on port ",string system "p"
