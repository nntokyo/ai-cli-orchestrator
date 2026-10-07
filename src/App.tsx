const pillars = [
  "Task-scoped provider sessions",
  "Safe provider failover",
  "Git-optional local workspaces",
  "macOS and Windows",
];

function App() {
  return (
    <main className="app-shell">
      <section className="hero" aria-labelledby="app-title">
        <p className="eyebrow">Bootstrap</p>
        <h1 id="app-title">AI CLI Orchestrator</h1>
        <p className="lead">
          A local desktop orchestrator for multiple AI coding CLIs.
        </p>
      </section>

      <section className="status-panel" aria-label="Bootstrap status">
        <div>
          <span className="status-dot" aria-hidden="true" />
          <strong>Application shell ready</strong>
        </div>
        <p>
          Provider routing, sessions, permissions, recovery, editor, and
          terminal features are implemented in their dedicated Issues.
        </p>
      </section>

      <ul className="pillars" aria-label="Project design pillars">
        {pillars.map((pillar) => (
          <li key={pillar}>{pillar}</li>
        ))}
      </ul>
    </main>
  );
}

export default App;
