import "./marketing.css";

const capabilities = [
  { number: "01", title: "A bigger view", body: "Bring your Android screen to your Windows workspace with a familiar, direct mirroring workflow.", tag: "SCREEN MIRRORING" },
  { number: "02", title: "Your network, your choice", body: "Connect over USB or use Android Wireless Debugging on a trusted local network.", tag: "FLEXIBLE CONNECTION" },
  { number: "03", title: "Permission stays visible", body: "Pairing, capture and control are designed around clear approval—not hidden access.", tag: "PRIVACY BY DESIGN" },
];

export default function MarketingSite() {
  return <main className="mx-site">
    <header className="site-nav">
      <a className="site-logo" href="#" aria-label="MirrorX home"><span className="site-logo-mark">M<span>.</span></span><span>mirror<span>x</span></span></a>
      <nav aria-label="Main navigation"><a href="#experience">Experience</a><a href="#security">Privacy</a><a href="#platforms">Platforms</a></nav>
      <a className="site-nav-cta" href="#get-started">Explore MirrorX <span>↗</span></a>
    </header>

    <section className="site-hero">
      <div className="site-hero-grid"/>
      <div className="site-hero-copy">
        <div className="site-eyebrow"><i/> A MORE CONNECTED WORKSPACE</div>
        <h1>Your phone.<br/><em>In its element.</em></h1>
        <p className="site-hero-lede">Bring your Android screen to your Windows desktop. A considered, local-first experience built to make the space between devices feel smaller.</p>
        <div className="site-hero-actions"><a className="site-button-primary" href="#get-started">Discover the experience <span>↗</span></a><a className="site-button-text" href="#experience">See what we're building <span>↓</span></a></div>
        <div className="site-proof"><span><i/> WINDOWS DESKTOP</span><span><i/> ANDROID COMPANION IN DEVELOPMENT</span><span><i/> LOCAL-FIRST DIRECTION</span></div>
      </div>
      <div className="site-art" aria-label="Illustration of a phone connected to a desktop workspace">
        <div className="site-art-ring ring-one"/><div className="site-art-ring ring-two"/>
        <div className="site-monitor"><div className="monitor-top"><span>mirrorx / workspace</span><span>•••</span></div><div className="monitor-content"><div className="monitor-side"><b/><i/><i/><i/><i/></div><div className="monitor-screen"><small>YOUR DEVICE, REIMAGINED</small><strong>One screen.<br/><em>More room.</em></strong><div className="monitor-screen-lines"><i/><i/><i/></div></div></div><div className="monitor-stand"/></div>
        <div className="site-phone"><div className="phone-camera"/><div className="phone-ui"><small>MIRRORX</small><b>Make room<br/>for more.</b><i/><i/><i/></div></div>
        <div className="site-float float-top"><span>↗</span><div><b>Direct by design</b><small>Local connection</small></div></div>
        <div className="site-float float-bottom"><span>◇</span><div><b>Consent first</b><small>You stay in control</small></div></div>
      </div>
      <div className="site-hero-foot"><span>01 — THE MIRRORX EXPERIENCE</span><span>SCROLL TO EXPLORE ↓</span></div>
    </section>

    <section className="site-manifesto" id="experience"><div className="site-section-index">A DIFFERENT KIND OF CONNECTION <span>— 01</span></div><div className="site-manifesto-grid"><h2>Less switching.<br/><em>More flow.</em></h2><div><p>Some things are better on a bigger screen. MirrorX is being built to make connecting your Android device to your Windows workspace feel deliberate, straightforward and dependable.</p><p className="site-muted">Not another cloud dashboard. Not a remote-access mystery. A device experience designed around the connection you choose.</p></div></div></section>

    <section className="site-capabilities"><div className="site-section-heading"><div><div className="site-section-index">THE FOUNDATION <span>— 02</span></div><h2>Thoughtful at<br/><em>every connection.</em></h2></div><p>One clear purpose, supported by practical details that matter when your devices work together.</p></div><div className="site-capability-grid">{capabilities.map((item) => <article className="site-capability" key={item.number}><div className="cap-top"><span>{item.number}</span><span>↗</span></div><div className="cap-tag">{item.tag}</div><h3>{item.title}</h3><p>{item.body}</p></article>)}</div></section>

    <section className="site-security" id="security"><div className="security-orb"/><div className="site-section-index">BUILT WITH INTENTION <span>— 03</span></div><div className="security-content"><div><div className="security-symbol">◇</div><h2>Your devices.<br/><em>Your permission.</em></h2></div><div className="security-copy"><p>Screen sharing and remote interaction deserve clarity. MirrorX is designed around explicit device approval, visible capture permission and the ability to end a session.</p><ul><li><span>01</span> Local-first media direction</li><li><span>02</span> Explicit capture approval</li><li><span>03</span> Separate control permission</li><li><span>04</span> Clear session revocation</li></ul><small>These are product design principles; full authenticated streaming and remote control are still under development.</small></div></div></section>

    <section className="site-platforms" id="platforms"><div className="site-section-index">THE PRODUCT ROADMAP <span>— 04</span></div><div className="platform-heading"><h2>One experience.<br/><em>Built across devices.</em></h2><p>We are building MirrorX in focused stages. Platform status is shown honestly, so you know what is available today and what is still being developed.</p></div><div className="platform-list"><article><span>01</span><div><h3>Windows desktop</h3><p>Device discovery and a scrcpy-based launch prototype.</p></div><b className="status-prototype">PROTOTYPE</b></article><article><span>02</span><div><h3>Android</h3><p>Screen capture and encoding foundation; live network streaming is not available yet.</p></div><b>IN DEVELOPMENT</b></article><article><span>03</span><div><h3>iOS</h3><p>ReplayKit capture and the iOS companion are planned, not available in this build.</p></div><b>PLANNED</b></article></div></section>

    <section className="site-cta" id="get-started"><div className="cta-glow"/><div className="site-section-index">THE NEXT STEP <span>— 05</span></div><h2>Make more room<br/><em>for what matters.</em></h2><p>MirrorX is under active development. Explore the project and follow its progress as the experience takes shape.</p><div className="site-hero-actions"><a className="site-button-primary" href="https://github.com/umer714/MirrorX" target="_blank" rel="noreferrer">Explore the project <span>↗</span></a><a className="site-button-text" href="https://github.com/umer714/MirrorX/issues" target="_blank" rel="noreferrer">Follow development <span>↗</span></a></div></section>

    <footer className="site-footer"><a className="site-logo" href="#"><span className="site-logo-mark">M<span>.</span></span><span>mirror<span>x</span></span></a><span>YOUR PHONE. YOUR PC. ONE EXPERIENCE.</span><div><a href="https://github.com/umer714/MirrorX" target="_blank" rel="noreferrer">GITHUB ↗</a><span>© 2026 MIRRORX</span></div></footer>
  </main>;
}
