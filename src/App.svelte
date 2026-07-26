<script lang="ts">
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import {
    ACTOR_BRIEF_LIMITS,
    actorBriefError,
    parseActorBrief,
  } from "./lib/studio/brief";
  import { BOUNDARY_RULES, TILEFORGE_ACTOR_CONTRACT } from "./lib/studio/contract";
  import { compileActorPrompt } from "./lib/studio/prompt";
  import { parseStudioSession } from "./lib/studio/session";
  import type { ActorBrief, StudioSession } from "./lib/studio/types";

  const stages = [
    { key: "brief", label: "Brief" },
    { key: "concept", label: "Concept" },
    { key: "turnaround", label: "Turnaround" },
    { key: "animate", label: "Animate" },
    { key: "world-test", label: "World test" },
    { key: "export", label: "Export" },
  ] as const;

  const scenes = ["Scale lineup", "Forest clearing", "Crownhold", "Tidewater"];
  const themes = ["Forest", "Autumn", "Dusk", "Winter"];

  let brief: ActorBrief = {
    name: "Mirelight Pilgrim",
    kind: "mob",
    description:
      "A small marsh pilgrim in a reed cloak, carrying a blue-green lantern. Quiet and strange rather than aggressive.",
  };
  let activeStage = 0;
  let selectedScene = scenes[0];
  let selectedTheme = themes[0];
  let session: StudioSession | null = null;
  let showCompiledPrompt = false;
  let sessionMessage = "Checking for saved sessions…";
  let sessionError = "";
  let workspaceRoot = "";
  let saving = false;
  let restoring = true;

  $: compiledPrompt = compileActorPrompt(brief);

  onMount(() => {
    void restoreLatestSession();
  });

  function showSession(saved: StudioSession) {
    session = saved;
    brief = { ...saved.brief };
    const restoredStage = stages.findIndex((stage) => stage.key === saved.stage);
    activeStage = restoredStage >= 0 ? restoredStage : 1;
  }

  async function restoreLatestSession() {
    sessionError = "";
    if (!isTauri()) {
      sessionMessage = "Browser preview only — durable sessions require the desktop app.";
      restoring = false;
      return;
    }

    try {
      const result = await invoke<{ workspaceRoot: string; sessions: unknown[] }>(
        "list_sprite_sessions",
      );
      workspaceRoot = result.workspaceRoot;
      const latest = result.sessions[0];
      if (latest) {
        showSession(parseStudioSession(latest));
        sessionMessage = "Reopened the latest durable session.";
      } else {
        sessionMessage = "No saved sessions yet.";
      }
    } catch (error) {
      sessionError = actorBriefError(error);
      sessionMessage = "Saved sessions could not be loaded.";
    } finally {
      restoring = false;
    }
  }

  async function beginSession() {
    sessionError = "";
    let validatedBrief: ActorBrief;
    try {
      validatedBrief = parseActorBrief(brief);
    } catch (error) {
      sessionError = actorBriefError(error);
      return;
    }

    if (!isTauri()) {
      sessionError = "Open the desktop app to create a durable session.";
      return;
    }

    saving = true;
    sessionMessage = "Saving a new immutable session…";
    try {
      const saved = parseStudioSession(
        await invoke("create_sprite_session", { brief: validatedBrief }),
      );
      showSession(saved);
      workspaceRoot ||= ".studio";
      sessionMessage = "Saved locally. MCP clients can read this session.";
    } catch (error) {
      sessionError = actorBriefError(error);
      sessionMessage = "The session was not saved.";
    } finally {
      saving = false;
    }
  }
</script>

<svelte:head>
  <meta name="theme-color" content="#111713" />
</svelte:head>

<main class="studio-shell">
  <header class="topbar">
    <div class="brand">
      <div class="brand-mark" aria-hidden="true">
        <span></span><span></span><span></span><span></span>
      </div>
      <div>
        <p class="eyebrow">TileForge</p>
        <h1>Actor Studio</h1>
      </div>
    </div>

    <div class="project-status">
      <span class="status-dot"></span>
      <span>{session ? session.brief.name : "New actor"}</span>
      <span class="revision">{session ? `r${session.revision}` : "unsaved"}</span>
    </div>
  </header>

  <nav class="stagebar" aria-label="Actor workflow">
    {#each stages as stage, index}
      <button
        class:active={activeStage === index}
        class:complete={activeStage > index}
        onclick={() => (activeStage = index)}
      >
        <span class="stage-index">{activeStage > index ? "✓" : index + 1}</span>
        <span>{stage.label}</span>
      </button>
    {/each}
  </nav>

  <section class="workspace">
    <aside class="brief-panel panel">
      <div class="panel-heading">
        <div>
          <p class="eyebrow">Creative input</p>
          <h2>Actor brief</h2>
        </div>
        <span class="panel-number">01</span>
      </div>

      <label>
        <span>Name</span>
        <input
          bind:value={brief.name}
          maxlength={ACTOR_BRIEF_LIMITS.nameMaxLength}
          placeholder="Name this actor"
        />
      </label>

      <fieldset>
        <legend>Actor type</legend>
        <div class="segmented">
          <button class:active={brief.kind === "mob"} onclick={() => (brief.kind = "mob")}>
            Mob
          </button>
          <button class:active={brief.kind === "npc"} onclick={() => (brief.kind = "npc")}>
            NPC
          </button>
        </div>
      </fieldset>

      <label class="grow">
        <span>Description</span>
        <textarea
          bind:value={brief.description}
          maxlength={ACTOR_BRIEF_LIMITS.descriptionMaxLength}
          rows="8"
        ></textarea>
        <small>Describe identity and mood. The world rules stay locked.</small>
      </label>

      {#if session}
        <section class="session-identity" aria-label="Saved session identity">
          <div class="saved-heading">
            <span class="status-dot"></span>
            <strong>Durable session</strong>
          </div>
          <dl>
            <div><dt>Session</dt><dd title={session.id}>{session.id}</dd></div>
            <div><dt>Revision</dt><dd>r{session.revision}</dd></div>
            <div><dt>Contract</dt><dd>{session.contractId}</dd></div>
          </dl>
          {#if workspaceRoot}
            <small>Saved under {workspaceRoot}</small>
          {/if}
        </section>
      {/if}

      <p class="session-message" class:error={Boolean(sessionError)} role={sessionError ? "alert" : "status"}>
        {sessionError || sessionMessage}
      </p>

      <button class="prompt-preview" onclick={() => (showCompiledPrompt = !showCompiledPrompt)}>
        <span>{showCompiledPrompt ? "Hide" : "Preview"} compiled AI prompt</span>
        <span aria-hidden="true">{showCompiledPrompt ? "−" : "+"}</span>
      </button>

      {#if showCompiledPrompt}
        <pre class="compiled-prompt">{compiledPrompt}</pre>
      {/if}

      <button class="primary-action" disabled={saving || restoring} onclick={beginSession}>
        <span>
          {saving
            ? "Saving…"
            : restoring
              ? "Loading sessions…"
              : session
                ? "Save as new concept"
                : "Begin concept"}
        </span>
        <span aria-hidden="true">→</span>
      </button>
    </aside>

    <section class="preview-panel panel">
      <div class="panel-heading preview-heading">
        <div>
          <p class="eyebrow">Approval surface</p>
          <h2>{activeStage === 0 ? "Concept preview" : stages[activeStage].label}</h2>
        </div>
        <div class="zoom-control" aria-label="Preview zoom">
          <button>1×</button>
          <button class="active">8×</button>
          <button>16×</button>
        </div>
      </div>

      <div class="preview-stage">
        <div class="canvas-frame">
          <div class="pixel-grid">
            <div class="sprite-placeholder" aria-label="Waiting for generated actor">
              <div class="sprite-head"></div>
              <div class="sprite-cloak"></div>
              <div class="sprite-lantern"></div>
              <div class="sprite-feet"></div>
            </div>
            <span class="anchor-line horizontal"></span>
            <span class="anchor-line vertical"></span>
            <span class="anchor-point" title="Foot anchor"></span>
          </div>
        </div>
        <p class="preview-note">
          {session
            ? "Concept slot ready. Generation will create immutable candidates here."
            : "Begin a concept to create the first immutable candidate."}
        </p>
      </div>

      <div class="world-strip">
        <div class="world-controls">
          <label>
            <span>Scene</span>
            <select bind:value={selectedScene}>
              {#each scenes as scene}<option>{scene}</option>{/each}
            </select>
          </label>
          <label>
            <span>Theme</span>
            <select bind:value={selectedTheme}>
              {#each themes as theme}<option>{theme}</option>{/each}
            </select>
          </label>
        </div>
        <div class="world-swatch" class:dusk={selectedTheme === "Dusk"} class:winter={selectedTheme === "Winter"}>
          <span class="terrain-grain one"></span>
          <span class="terrain-grain two"></span>
          <span class="terrain-grain three"></span>
          <div class="world-actor">
            <span></span>
          </div>
          <small>{selectedScene} · {selectedTheme}</small>
        </div>
      </div>
    </section>

    <aside class="contract-panel panel">
      <div class="panel-heading">
        <div>
          <p class="eyebrow">Immutable rules</p>
          <h2>World contract</h2>
        </div>
        <span class="lock-mark" aria-label="Locked">◆</span>
      </div>

      <div class="contract-id">
        <span>{TILEFORGE_ACTOR_CONTRACT.title}</span>
        <code>v{TILEFORGE_ACTOR_CONTRACT.version}</code>
      </div>

      <div class="rules">
        {#each BOUNDARY_RULES as rule}
          <div class="rule">
            <span class:warning={rule.severity === "warning"} class="rule-state">
              {rule.severity === "locked" ? "✓" : "!"}
            </span>
            <div>
              <strong>{rule.label}</strong>
              <span>{rule.value}</span>
            </div>
          </div>
        {/each}
      </div>

      <div class="approval-card">
        <p class="eyebrow">Approval boundary</p>
        <strong>Only you can approve final art.</strong>
        <span>Connected agents may generate, compare, validate, and prepare exports.</span>
      </div>

      <div class="agent-row">
        <span>Agent gateway</span>
        <div class="agent-badges" aria-label="Supported AI clients">
          <span>Codex</span>
          <span>Claude</span>
          <span>Antigravity</span>
        </div>
      </div>
    </aside>
  </section>
</main>
