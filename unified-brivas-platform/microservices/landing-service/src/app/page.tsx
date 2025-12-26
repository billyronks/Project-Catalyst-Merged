export default function Home() {
    return (
        <main className="min-h-screen bg-gradient-to-br from-slate-900 via-purple-900 to-slate-900">
            {/* Hero Section */}
            <section className="container mx-auto px-6 py-20">
                <nav className="flex justify-between items-center mb-16">
                    <div className="text-2xl font-bold text-white">Brivas</div>
                    <div className="flex gap-8">
                        <a href="#features" className="text-gray-300 hover:text-white transition">Features</a>
                        <a href="#pricing" className="text-gray-300 hover:text-white transition">Pricing</a>
                        <a href="#docs" className="text-gray-300 hover:text-white transition">Docs</a>
                        <a href="/login" className="bg-purple-600 px-6 py-2 rounded-lg text-white hover:bg-purple-700 transition">Login</a>
                    </div>
                </nav>

                <div className="text-center max-w-4xl mx-auto">
                    <h1 className="text-6xl font-bold text-white mb-6 leading-tight">
                        Unified Messaging<br />
                        <span className="bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
                            For Every Channel
                        </span>
                    </h1>
                    <p className="text-xl text-gray-300 mb-10">
                        SMS, WhatsApp, USSD, Telegram, and 16+ messaging platforms.<br />
                        One API. Carrier-grade reliability. 100,000+ TPS.
                    </p>
                    <div className="flex gap-4 justify-center">
                        <a href="/signup" className="bg-purple-600 px-8 py-4 rounded-xl text-white font-semibold hover:bg-purple-700 transition text-lg">
                            Start Free Trial
                        </a>
                        <a href="/docs" className="border border-gray-500 px-8 py-4 rounded-xl text-white font-semibold hover:border-white transition text-lg">
                            View Documentation
                        </a>
                    </div>
                </div>
            </section>

            {/* Features Section */}
            <section id="features" className="container mx-auto px-6 py-20">
                <h2 className="text-4xl font-bold text-white text-center mb-16">Platform Features</h2>
                <div className="grid md:grid-cols-3 gap-8">
                    {[
                        { title: 'Multi-Channel', desc: 'SMS, WhatsApp, USSD, Telegram, Slack, Discord, and 10+ more', icon: '📱' },
                        { title: 'High Performance', desc: '100,000+ TPS with <50ms P99 latency. Carrier-grade reliability.', icon: '⚡' },
                        { title: 'AI-Powered', desc: 'LLM integration for smart routing, fraud detection, and automation.', icon: '🤖' },
                        { title: 'SMPP Server', desc: 'Full SMPP v3.4/5.0 support for carrier integrations.', icon: '🔌' },
                        { title: 'USSD Gateway', desc: 'Dynamic menus, session management, multi-operator support.', icon: '📞' },
                        { title: 'Payment Integration', desc: 'Paystack, Flutterwave, and wallet management built-in.', icon: '💳' },
                    ].map((feature, i) => (
                        <div key={i} className="bg-white/5 border border-white/10 rounded-2xl p-8 hover:bg-white/10 transition">
                            <div className="text-4xl mb-4">{feature.icon}</div>
                            <h3 className="text-xl font-semibold text-white mb-2">{feature.title}</h3>
                            <p className="text-gray-400">{feature.desc}</p>
                        </div>
                    ))}
                </div>
            </section>

            {/* Stats Section */}
            <section className="container mx-auto px-6 py-20">
                <div className="grid md:grid-cols-4 gap-8 text-center">
                    {[
                        { value: '100K+', label: 'Messages/Second' },
                        { value: '16+', label: 'Platforms' },
                        { value: '99.99%', label: 'Uptime SLA' },
                        { value: '<50ms', label: 'P99 Latency' },
                    ].map((stat, i) => (
                        <div key={i}>
                            <div className="text-5xl font-bold bg-gradient-to-r from-purple-400 to-pink-400 bg-clip-text text-transparent">
                                {stat.value}
                            </div>
                            <div className="text-gray-400 mt-2">{stat.label}</div>
                        </div>
                    ))}
                </div>
            </section>

            {/* Footer */}
            <footer className="container mx-auto px-6 py-12 border-t border-white/10">
                <div className="flex justify-between items-center">
                    <div className="text-gray-400">© 2024 Brivas. All rights reserved.</div>
                    <div className="flex gap-6">
                        <a href="/privacy" className="text-gray-400 hover:text-white">Privacy</a>
                        <a href="/terms" className="text-gray-400 hover:text-white">Terms</a>
                        <a href="/contact" className="text-gray-400 hover:text-white">Contact</a>
                    </div>
                </div>
            </footer>
        </main>
    );
}
