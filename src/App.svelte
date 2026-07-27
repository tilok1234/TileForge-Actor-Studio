<script lang="ts">
  import { invoke, isTauri } from "@tauri-apps/api/core";
  import { onDestroy, onMount } from "svelte";
  import {
    ACTOR_BRIEF_LIMITS,
    actorBriefError,
    parseActorBrief,
  } from "./lib/studio/brief";
  import {
    CONCEPT_PNG_MAX_BYTES,
    parseConceptCandidate,
    type ConceptCandidate,
  } from "./lib/studio/candidate";
  import { BOUNDARY_RULES, TILEFORGE_ACTOR_CONTRACT } from "./lib/studio/contract";
  import { compileActorPrompt } from "./lib/studio/prompt";
  import { parseStudioSession } from "./lib/studio/session";
  import {
    parseTurnaroundCandidate,
    TURNAROUND_DIRECTIONS,
    type TurnaroundCandidate,
    type TurnaroundDirection,
  } from "./lib/studio/turnaround";
  import {
    parseTurnaroundValidationReport,
    type TurnaroundValidationReport,
  } from "./lib/studio/turnaround-validation";
  import {
    parseWalkCycleCandidate,
    WALK_CYCLE_FRAMES_PER_DIRECTION,
    type WalkCycleCandidate,
  } from "./lib/studio/walk-cycle";
  import {
    parseWalkCycleValidationReport,
    type WalkCycleValidationReport,
  } from "./lib/studio/walk-cycle-validation";
  import {
    type WorldTestScene,
    type WorldTestTheme,
  } from "./lib/studio/reference-pack";
  import {
    parseWorldTestCandidate,
    type WorldTestCandidate,
  } from "./lib/studio/world-test";
  import {
    parseWorldTestValidationReport,
    type WorldTestValidationReport,
  } from "./lib/studio/world-test-validation";
  import type { ActorBrief, StudioSession } from "./lib/studio/types";
  import {
    parseValidationReport,
    type ValidationReport,
    type ValidationRuleResult,
  } from "./lib/studio/validation";

  const stages = [
    { key: "brief", label: "Brief" },
    { key: "concept", label: "Concept" },
    { key: "turnaround", label: "Turnaround" },
    { key: "animate", label: "Animate" },
    { key: "world-test", label: "World test" },
    { key: "export", label: "Export" },
  ] as const;

  const scenes = ["Scale lineup", "Forest clearing", "Crownhold", "Tidewater"] as const;
  const themes = ["Forest", "Autumn", "Dusk", "Winter"] as const;
  const sceneIds: Record<(typeof scenes)[number], WorldTestScene> = {
    "Scale lineup": "scale-lineup",
    "Forest clearing": "forest-clearing",
    Crownhold: "crownhold",
    Tidewater: "tidewater",
  };
  const themeIds: Record<(typeof themes)[number], WorldTestTheme> = {
    Forest: "forest",
    Autumn: "autumn",
    Dusk: "dusk",
    Winter: "winter",
  };
  const validationLabels: Record<ValidationRuleResult["id"], string> = {
    canvas_dimensions: "Canvas",
    hard_alpha: "Hard alpha",
    actor_height: "Actor height",
    foot_anchor: "Foot anchor",
    palette_max_colors: "Palette",
    ground_luma_separation: "Ground contrast",
    frame_edge_clipping: "Frame edge",
  };

  let brief: ActorBrief = {
    name: "Mirelight Pilgrim",
    kind: "mob",
    description:
      "A small marsh pilgrim in a reed cloak, carrying a blue-green lantern. Quiet and strange rather than aggressive.",
  };
  let activeStage = 0;
  let selectedScene: (typeof scenes)[number] = scenes[0];
  let selectedTheme: (typeof themes)[number] = themes[0];
  let session: StudioSession | null = null;
  let showCompiledPrompt = false;
  let sessionMessage = "Checking for saved sessions…";
  let sessionError = "";
  let workspaceRoot = "";
  let saving = false;
  let restoring = true;
  let candidates: ConceptCandidate[] = [];
  let selectedCandidate: ConceptCandidate | null = null;
  let candidatePngUrl = "";
  let candidateMessage = "Import a 32 × 32 PNG to create the first immutable candidate.";
  let candidateError = "";
  let importingCandidate = false;
  let previewZoom: 1 | 8 | 16 = 8;
  let validationReport: ValidationReport | null = null;
  let validationError = "";
  let validating = false;
  let turnarounds: TurnaroundCandidate[] = [];
  let selectedTurnaround: TurnaroundCandidate | null = null;
  let turnaroundPngUrls: Record<TurnaroundDirection, string> = {
    down: "",
    right: "",
    up: "",
    left: "",
  };
  let turnaroundMessage = "No immutable Turnaround has been created yet.";
  let turnaroundError = "";
  let turnaroundValidation: TurnaroundValidationReport | null = null;
  let validatingTurnaround = false;
  let walkCycles: WalkCycleCandidate[] = [];
  let selectedWalkCycle: WalkCycleCandidate | null = null;
  let walkCyclePngUrls: Record<TurnaroundDirection, string[]> = {
    down: [],
    right: [],
    up: [],
    left: [],
  };
  let walkCycleMessage = "No immutable Walk Cycle has been created yet.";
  let walkCycleError = "";
  let walkCycleValidation: WalkCycleValidationReport | null = null;
  let validatingWalkCycle = false;
  let animationFrameIndex = 0;
  let animationTimer: number | undefined;
  let worldTests: WorldTestCandidate[] = [];
  let selectedWorldTest: WorldTestCandidate | null = null;
  let worldTestPreviewUrls: Record<string, string> = {};
  let worldTestMessage = "No immutable World Test has been prepared yet.";
  let worldTestError = "";
  let worldTestValidation: WorldTestValidationReport | null = null;
  let validatingWorldTest = false;
  let creatingWorldTest = false;

  $: compiledPrompt = compileActorPrompt(brief);
  $: selectedWorldPreviewKey = `${sceneIds[selectedScene]}/${themeIds[selectedTheme]}`;
  $: selectedWorldPreviewUrl =
    worldTestPreviewUrls[selectedWorldPreviewKey] ?? "";
  $: selectedWorldGroundSummary =
    worldTestValidation?.measurements
      .filter(
        (measurement) =>
          measurement.scene === sceneIds[selectedScene] &&
          measurement.theme === themeIds[selectedTheme],
      )
      .reduce(
        (summary, measurement) => {
          summary[measurement.status] += 1;
          return summary;
        },
        { pass: 0, fail: 0 },
      ) ?? { pass: 0, fail: 0 };

  onMount(() => {
    void restoreLatestSession();
    animationTimer = window.setInterval(() => {
      animationFrameIndex =
        (animationFrameIndex + 1) % WALK_CYCLE_FRAMES_PER_DIRECTION;
    }, TILEFORGE_ACTOR_CONTRACT.animation.frameDurationMs);
  });

  onDestroy(() => {
    revokeCandidateUrl();
    revokeTurnaroundUrls();
    revokeWalkCycleUrls();
    revokeWorldTestUrls();
    if (animationTimer !== undefined) {
      window.clearInterval(animationTimer);
    }
  });

  function revokeCandidateUrl() {
    if (candidatePngUrl) {
      URL.revokeObjectURL(candidatePngUrl);
      candidatePngUrl = "";
    }
  }

  function clearCandidates() {
    revokeCandidateUrl();
    candidates = [];
    selectedCandidate = null;
    validationReport = null;
    validationError = "";
    candidateError = "";
    candidateMessage = "Import a 32 × 32 PNG to create the first immutable candidate.";
  }

  function revokeTurnaroundUrls() {
    for (const direction of TURNAROUND_DIRECTIONS) {
      if (turnaroundPngUrls[direction]) {
        URL.revokeObjectURL(turnaroundPngUrls[direction]);
      }
    }
    turnaroundPngUrls = { down: "", right: "", up: "", left: "" };
  }

  function clearTurnarounds() {
    revokeTurnaroundUrls();
    turnarounds = [];
    selectedTurnaround = null;
    turnaroundValidation = null;
    turnaroundError = "";
    turnaroundMessage = "No immutable Turnaround has been created yet.";
  }

  function revokeWalkCycleUrls() {
    for (const direction of TURNAROUND_DIRECTIONS) {
      for (const url of walkCyclePngUrls[direction]) {
        URL.revokeObjectURL(url);
      }
    }
    walkCyclePngUrls = { down: [], right: [], up: [], left: [] };
  }

  function clearWalkCycles() {
    revokeWalkCycleUrls();
    walkCycles = [];
    selectedWalkCycle = null;
    walkCycleValidation = null;
    walkCycleError = "";
    walkCycleMessage = "No immutable Walk Cycle has been created yet.";
  }

  function revokeWorldTestUrls() {
    for (const url of Object.values(worldTestPreviewUrls)) {
      URL.revokeObjectURL(url);
    }
    worldTestPreviewUrls = {};
  }

  function clearWorldTests() {
    revokeWorldTestUrls();
    worldTests = [];
    selectedWorldTest = null;
    worldTestValidation = null;
    worldTestError = "";
    worldTestMessage = "No immutable World Test has been prepared yet.";
  }

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
        const restoredSession = parseStudioSession(latest);
        showSession(restoredSession);
        await refreshCandidates(restoredSession.id);
        await refreshTurnarounds(restoredSession.id);
        await refreshWalkCycles(restoredSession.id);
        await refreshWorldTests(restoredSession.id);
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
      clearCandidates();
      clearTurnarounds();
      clearWalkCycles();
      clearWorldTests();
      workspaceRoot ||= ".studio";
      sessionMessage = "Saved locally. MCP clients can read this session.";
    } catch (error) {
      sessionError = actorBriefError(error);
      sessionMessage = "The session was not saved.";
    } finally {
      saving = false;
    }
  }

  async function refreshCandidates(sessionId: string, preferredId?: string) {
    candidateError = "";
    try {
      const result = await invoke<unknown[]>("list_concept_candidates", {
        sessionId,
      });
      candidates = result.map(parseConceptCandidate);
      const preferred =
        candidates.find((candidate) => candidate.id === preferredId) ??
        candidates[0] ??
        null;
      if (preferred) {
        await loadCandidate(preferred);
        candidateMessage = `${candidates.length} immutable candidate${candidates.length === 1 ? "" : "s"} saved.`;
      } else {
        clearCandidates();
      }
    } catch (error) {
      candidateError = actorBriefError(error);
      candidateMessage = "Candidates could not be loaded.";
    }
  }

  async function loadCandidate(candidate: ConceptCandidate) {
    if (!session || !isTauri()) {
      return;
    }
    candidateError = "";
    validationReport = null;
    validationError = "";
    try {
      const payload = await invoke<{ candidate: unknown; pngBytes: number[] }>(
        "get_concept_candidate",
        {
          sessionId: session.id,
          candidateId: candidate.id,
        },
      );
      const loaded = parseConceptCandidate(payload.candidate);
      const bytes = new Uint8Array(payload.pngBytes);
      revokeCandidateUrl();
      candidatePngUrl = URL.createObjectURL(
        new Blob([bytes.buffer], { type: loaded.mimeType }),
      );
      selectedCandidate = loaded;
      await validateCandidate(loaded);
    } catch (error) {
      candidateError = actorBriefError(error);
    }
  }

  async function validateCandidate(candidate: ConceptCandidate) {
    if (!session || !isTauri()) {
      return;
    }
    validating = true;
    validationError = "";
    try {
      const report = parseValidationReport(
        await invoke("validate_concept_candidate", {
          sessionId: session.id,
          candidateId: candidate.id,
        }),
      );
      if (
        report.candidateId !== candidate.id ||
        report.candidateSha256 !== candidate.sha256
      ) {
        throw new Error("Validation report identity does not match the candidate.");
      }
      validationReport = report;
    } catch (error) {
      validationReport = null;
      validationError = actorBriefError(error);
    } finally {
      validating = false;
    }
  }

  async function importCandidate(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file || !session) {
      return;
    }
    candidateError = "";
    if (!isTauri()) {
      candidateError = "Open the desktop app to import a durable candidate.";
      return;
    }
    if (file.size > CONCEPT_PNG_MAX_BYTES) {
      candidateError = "PNG file exceeds the 1 MiB intake limit.";
      return;
    }

    importingCandidate = true;
    candidateMessage = "Validating and preserving original PNG bytes…";
    try {
      const saved = parseConceptCandidate(
        await invoke("import_concept_candidate", {
          sessionId: session.id,
          pngBytes: Array.from(new Uint8Array(await file.arrayBuffer())),
          provenance: {
            source: "imported",
            originalFilename: file.name,
          },
        }),
      );
      await refreshCandidates(session.id, saved.id);
      candidateMessage =
        "Structural intake passed. Visual judgment is not assessed; candidate remains unreviewed.";
    } catch (error) {
      candidateError = actorBriefError(error);
      candidateMessage = "Candidate was rejected without creating a partial revision.";
    } finally {
      importingCandidate = false;
    }
  }

  async function refreshTurnarounds(
    sessionId: string,
    preferredId?: string,
  ) {
    turnaroundError = "";
    try {
      const result = await invoke<unknown[]>("list_turnaround_candidates", {
        sessionId,
      });
      turnarounds = result.map(parseTurnaroundCandidate);
      const preferred =
        turnarounds.find((candidate) => candidate.id === preferredId) ??
        turnarounds[0] ??
        null;
      if (preferred) {
        await loadTurnaround(preferred);
        turnaroundMessage = `${turnarounds.length} immutable Turnaround revision${turnarounds.length === 1 ? "" : "s"} saved.`;
        activeStage = 2;
      } else {
        clearTurnarounds();
      }
    } catch (error) {
      turnaroundError = actorBriefError(error);
      turnaroundMessage = "Turnarounds could not be loaded.";
    }
  }

  async function loadTurnaround(candidate: TurnaroundCandidate) {
    if (!session || !isTauri()) {
      return;
    }
    turnaroundError = "";
    turnaroundValidation = null;
    try {
      const payload = await invoke<{
        candidate: unknown;
        pngBytes: Record<TurnaroundDirection, number[]>;
      }>("get_turnaround_candidate", {
        sessionId: session.id,
        turnaroundId: candidate.id,
      });
      const loaded = parseTurnaroundCandidate(payload.candidate);
      const sourceCandidate = candidates.find(
        (concept) => concept.id === loaded.sourceSelection.candidateId,
      );
      if (sourceCandidate && selectedCandidate?.id !== sourceCandidate.id) {
        await loadCandidate(sourceCandidate);
      }
      revokeTurnaroundUrls();
      turnaroundPngUrls = Object.fromEntries(
        TURNAROUND_DIRECTIONS.map((direction) => [
          direction,
          URL.createObjectURL(
            new Blob(
              [new Uint8Array(payload.pngBytes[direction]).buffer],
              { type: "image/png" },
            ),
          ),
        ]),
      ) as Record<TurnaroundDirection, string>;
      selectedTurnaround = loaded;
      await validateTurnaround(loaded);
    } catch (error) {
      turnaroundError = actorBriefError(error);
    }
  }

  async function validateTurnaround(candidate: TurnaroundCandidate) {
    if (!session || !isTauri()) {
      return;
    }
    validatingTurnaround = true;
    turnaroundError = "";
    try {
      const report = parseTurnaroundValidationReport(
        await invoke("validate_turnaround_candidate", {
          sessionId: session.id,
          turnaroundId: candidate.id,
        }),
      );
      if (report.turnaroundId !== candidate.id) {
        throw new Error(
          "Turnaround validation identity does not match the candidate.",
        );
      }
      turnaroundValidation = report;
    } catch (error) {
      turnaroundValidation = null;
      turnaroundError = actorBriefError(error);
    } finally {
      validatingTurnaround = false;
    }
  }

  async function refreshWalkCycles(
    sessionId: string,
    preferredId?: string,
  ) {
    walkCycleError = "";
    try {
      const result = await invoke<unknown[]>("list_walk_cycle_candidates", {
        sessionId,
      });
      walkCycles = result.map(parseWalkCycleCandidate);
      const preferred =
        walkCycles.find((candidate) => candidate.id === preferredId) ??
        walkCycles[0] ??
        null;
      if (preferred) {
        await loadWalkCycle(preferred);
        walkCycleMessage = `${walkCycles.length} immutable Walk Cycle revision${walkCycles.length === 1 ? "" : "s"} saved.`;
        activeStage = 3;
      } else {
        clearWalkCycles();
      }
    } catch (error) {
      walkCycleError = actorBriefError(error);
      walkCycleMessage = "Walk Cycles could not be loaded.";
    }
  }

  async function loadWalkCycle(candidate: WalkCycleCandidate) {
    if (!session || !isTauri()) {
      return;
    }
    walkCycleError = "";
    walkCycleValidation = null;
    try {
      const payload = await invoke<{
        candidate: unknown;
        pngBytes: Record<TurnaroundDirection, number[][]>;
      }>("get_walk_cycle_candidate", {
        sessionId: session.id,
        walkCycleId: candidate.id,
      });
      const loaded = parseWalkCycleCandidate(payload.candidate);
      const sourceTurnaround = turnarounds.find(
        (turnaround) =>
          turnaround.id === loaded.sourceTurnaround.turnaroundId,
      );
      if (
        sourceTurnaround &&
        selectedTurnaround?.id !== sourceTurnaround.id
      ) {
        await loadTurnaround(sourceTurnaround);
      }
      revokeWalkCycleUrls();
      walkCyclePngUrls = Object.fromEntries(
        TURNAROUND_DIRECTIONS.map((direction) => [
          direction,
          payload.pngBytes[direction].map((bytes) =>
            URL.createObjectURL(
              new Blob([new Uint8Array(bytes).buffer], {
                type: "image/png",
              }),
            ),
          ),
        ]),
      ) as Record<TurnaroundDirection, string[]>;
      selectedWalkCycle = loaded;
      animationFrameIndex = 0;
      await validateWalkCycle(loaded);
    } catch (error) {
      walkCycleError = actorBriefError(error);
    }
  }

  async function validateWalkCycle(candidate: WalkCycleCandidate) {
    if (!session || !isTauri()) {
      return;
    }
    validatingWalkCycle = true;
    walkCycleError = "";
    try {
      const report = parseWalkCycleValidationReport(
        await invoke("validate_walk_cycle_candidate", {
          sessionId: session.id,
          walkCycleId: candidate.id,
        }),
      );
      if (report.walkCycleId !== candidate.id) {
        throw new Error(
          "Walk Cycle validation identity does not match the candidate.",
        );
      }
      walkCycleValidation = report;
    } catch (error) {
      walkCycleValidation = null;
      walkCycleError = actorBriefError(error);
    } finally {
      validatingWalkCycle = false;
    }
  }

  function walkDirectionSummary(direction: TurnaroundDirection) {
    return (
      walkCycleValidation?.frames
        .filter((frame) => frame.direction === direction)
        .reduce(
          (summary, frame) => ({
            pass: summary.pass + frame.report.summary.pass,
            fail: summary.fail + frame.report.summary.fail,
            notAssessed:
              summary.notAssessed + frame.report.summary.notAssessed,
          }),
          { pass: 0, fail: 0, notAssessed: 0 },
        ) ?? { pass: 0, fail: 0, notAssessed: 0 }
    );
  }

  async function refreshWorldTests(
    sessionId: string,
    preferredId?: string,
  ) {
    worldTestError = "";
    try {
      const result = await invoke<unknown[]>("list_world_test_candidates", {
        sessionId,
      });
      worldTests = result.map(parseWorldTestCandidate);
      const preferred =
        worldTests.find((candidate) => candidate.id === preferredId) ??
        worldTests[0] ??
        null;
      if (preferred) {
        await loadWorldTest(preferred);
        worldTestMessage = `${worldTests.length} immutable World Test revision${worldTests.length === 1 ? "" : "s"} saved.`;
        activeStage = 4;
      } else {
        clearWorldTests();
      }
    } catch (error) {
      worldTestError = actorBriefError(error);
      worldTestMessage = "World Tests could not be loaded.";
    }
  }

  async function loadWorldTest(candidate: WorldTestCandidate) {
    if (!session || !isTauri()) {
      return;
    }
    worldTestError = "";
    worldTestValidation = null;
    try {
      const payload = await invoke<{
        candidate: unknown;
        previewPngBytes: Record<string, number[]>;
      }>("get_world_test_candidate", {
        sessionId: session.id,
        worldTestId: candidate.id,
      });
      const loaded = parseWorldTestCandidate(payload.candidate);
      const sourceWalkCycle = walkCycles.find(
        (walkCycle) =>
          walkCycle.id === loaded.sourceWalkCycle.walkCycleId,
      );
      if (sourceWalkCycle && selectedWalkCycle?.id !== sourceWalkCycle.id) {
        await loadWalkCycle(sourceWalkCycle);
      }
      revokeWorldTestUrls();
      worldTestPreviewUrls = Object.fromEntries(
        loaded.previews.map((preview) => {
          const key = `${preview.scene}/${preview.theme}`;
          const bytes = payload.previewPngBytes[key];
          if (!bytes) {
            throw new Error(`World Test preview ${key} is missing.`);
          }
          return [
            key,
            URL.createObjectURL(
              new Blob([new Uint8Array(bytes).buffer], {
                type: "image/png",
              }),
            ),
          ];
        }),
      );
      selectedWorldTest = loaded;
      await validateWorldTest(loaded);
    } catch (error) {
      worldTestError = actorBriefError(error);
    }
  }

  async function validateWorldTest(candidate: WorldTestCandidate) {
    if (!session || !isTauri()) {
      return;
    }
    validatingWorldTest = true;
    worldTestError = "";
    try {
      const report = parseWorldTestValidationReport(
        await invoke("validate_world_test_candidate", {
          sessionId: session.id,
          worldTestId: candidate.id,
        }),
      );
      if (report.worldTestId !== candidate.id) {
        throw new Error(
          "World Test validation identity does not match the candidate.",
        );
      }
      worldTestValidation = report;
    } catch (error) {
      worldTestValidation = null;
      worldTestError = actorBriefError(error);
    } finally {
      validatingWorldTest = false;
    }
  }

  async function prepareWorldTest() {
    if (!session || !selectedWalkCycle || !isTauri()) {
      return;
    }
    creatingWorldTest = true;
    worldTestError = "";
    worldTestMessage =
      "Preparing sixteen immutable previews from the pinned reference pack…";
    try {
      const saved = parseWorldTestCandidate(
        await invoke("create_world_test_candidate", {
          sessionId: session.id,
          sourceWalkCycleId: selectedWalkCycle.id,
        }),
      );
      await refreshWorldTests(session.id, saved.id);
      activeStage = 4;
      worldTestMessage =
        "World Test prepared locally. Final-art approval remains yours alone.";
    } catch (error) {
      worldTestError = actorBriefError(error);
      worldTestMessage =
        "World Test preparation failed without creating a partial revision.";
    } finally {
      creatingWorldTest = false;
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

        <section class="candidate-intake" aria-label="Concept candidate intake">
          <div>
            <p class="eyebrow">Concept intake</p>
            <strong>Immutable PNG import</strong>
          </div>
          <label class="secondary-action" class:disabled={importingCandidate}>
            <span>{importingCandidate ? "Importing…" : "Import 32 × 32 PNG"}</span>
            <input
              accept="image/png,.png"
              disabled={importingCandidate}
              onchange={importCandidate}
              type="file"
            />
          </label>
          <small>Original bytes and provenance are preserved. Import never approves art.</small>
          <p class="candidate-message" class:error={Boolean(candidateError)} role={candidateError ? "alert" : "status"}>
            {candidateError || candidateMessage}
          </p>
        </section>

        {#if selectedTurnaround}
          <section class="turnaround-selection" aria-label="Turnaround source selection">
            <p class="eyebrow">User selection receipt</p>
            <strong>Concept r{candidates.find((candidate) => candidate.id === selectedTurnaround?.sourceSelection.candidateId)?.revision ?? "?"}</strong>
            <small title={selectedTurnaround.sourceSelection.candidateId}>
              {selectedTurnaround.sourceSelection.candidateId}
            </small>
            <span>Recorded for Turnaround by user authority. This is not final-art approval.</span>
          </section>
        {/if}

        {#if selectedWalkCycle}
          <section class="turnaround-selection walk-source" aria-label="Accepted Turnaround receipt">
            <p class="eyebrow">User acceptance receipt</p>
            <strong>Turnaround r{turnarounds.find((candidate) => candidate.id === selectedWalkCycle?.sourceTurnaround.turnaroundId)?.revision ?? "?"}</strong>
            <small title={selectedWalkCycle.sourceTurnaround.turnaroundId}>
              {selectedWalkCycle.sourceTurnaround.turnaroundId}
            </small>
            <span>Accepted for animation by user authority. Final-art and publishing approval remain separate.</span>
          </section>
        {/if}

        {#if activeStage >= 4 && selectedWalkCycle}
          <section class="candidate-intake world-test-action" aria-label="World Test preparation">
            <div>
              <p class="eyebrow">Pinned world evidence</p>
              <strong>TileForge reference pack v1</strong>
            </div>
            <button
              class="secondary-action"
              disabled={creatingWorldTest}
              onclick={prepareWorldTest}
            >
              {creatingWorldTest ? "Preparing…" : "Prepare new World Test"}
            </button>
            <small>Uses only local deterministic compositing. It records your accepted Walk Cycle but never approves final art.</small>
          </section>
        {/if}

        {#if selectedWorldTest}
          <section class="turnaround-selection world-source" aria-label="Accepted Walk Cycle receipt">
            <p class="eyebrow">User acceptance receipt</p>
            <strong>Walk Cycle r{walkCycles.find((candidate) => candidate.id === selectedWorldTest?.sourceWalkCycle.walkCycleId)?.revision ?? "?"}</strong>
            <small title={selectedWorldTest.sourceWalkCycle.walkCycleId}>
              {selectedWorldTest.sourceWalkCycle.walkCycleId}
            </small>
            <span>Accepted for World Test by user authority. Final-art and publishing approval remain separate.</span>
          </section>
        {/if}
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
          <button class:active={previewZoom === 1} onclick={() => (previewZoom = 1)}>1×</button>
          <button class:active={previewZoom === 8} onclick={() => (previewZoom = 8)}>8×</button>
          <button class:active={previewZoom === 16} onclick={() => (previewZoom = 16)}>16×</button>
        </div>
      </div>

      <div
        class:turnaround={activeStage === 2}
        class:animate={activeStage === 3}
        class:world-test={activeStage === 4}
        class="preview-stage"
      >
        {#if activeStage === 4 && selectedWorldTest && selectedWorldPreviewUrl}
          <figure class="world-test-canvas">
            <img
              alt={`${selectedScene} ${selectedTheme} World Test preview for revision ${selectedWorldTest.revision}`}
              src={selectedWorldPreviewUrl}
            />
            <figcaption>
              <span>{selectedScene} · {selectedTheme}</span>
              <small>1× ground truth · actor at 1×</small>
            </figcaption>
          </figure>
        {:else if activeStage === 4}
          <div class="world-test-empty">
            <strong>World Test is ready to prepare.</strong>
            <span>Select “Prepare new World Test” to preserve all sixteen pinned scene/theme previews.</span>
          </div>
        {:else if activeStage === 3 && selectedWalkCycle}
          <div
            class="walk-cycle-grid"
            style={`--walk-cycle-size: ${32 * Math.min(previewZoom, 8)}px`}
          >
            {#each TURNAROUND_DIRECTIONS as direction}
              <figure>
                <div class="walk-cycle-canvas">
                  <img
                    alt={`${direction} walk animation for revision ${selectedWalkCycle.revision}, frame ${animationFrameIndex + 1}`}
                    height={32 * Math.min(previewZoom, 8)}
                    src={walkCyclePngUrls[direction][animationFrameIndex]}
                    width={32 * Math.min(previewZoom, 8)}
                  />
                </div>
                <figcaption>
                  <span>{direction}</span>
                  <small>frame {animationFrameIndex + 1}/4</small>
                </figcaption>
              </figure>
            {/each}
          </div>
        {:else if activeStage === 2 && selectedTurnaround}
          <div
            class="turnaround-grid"
            style={`--turnaround-size: ${32 * Math.min(previewZoom, 8)}px`}
          >
            {#each TURNAROUND_DIRECTIONS as direction}
              <figure>
                <div class="turnaround-canvas">
                  <img
                    alt={`${direction} view for Turnaround revision ${selectedTurnaround.revision}`}
                    height={32 * Math.min(previewZoom, 8)}
                    src={turnaroundPngUrls[direction]}
                    width={32 * Math.min(previewZoom, 8)}
                  />
                </div>
                <figcaption>{direction}</figcaption>
              </figure>
            {/each}
          </div>
        {:else if selectedCandidate && candidatePngUrl}
          <div class="candidate-canvas" style={`--candidate-size: ${32 * previewZoom}px`}>
            <img
              alt={`Concept candidate revision ${selectedCandidate.revision}`}
              height={32 * previewZoom}
              src={candidatePngUrl}
              width={32 * previewZoom}
            />
          </div>
        {:else}
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
        {/if}
        <p class="preview-note">
          {activeStage === 4 && selectedWorldTest
            ? `World Test r${selectedWorldTest.revision} · 16 immutable previews · final art not assessed`
            : activeStage === 3 && selectedWalkCycle
            ? `Walk Cycle r${selectedWalkCycle.revision} · 4 × 4 immutable frames · ${selectedWalkCycle.frameDurationMs} ms · motion not assessed`
            : activeStage === 2 && selectedTurnaround
            ? `Turnaround r${selectedTurnaround.revision} · four immutable views · identity consistency not assessed`
            : selectedCandidate
            ? `Candidate r${selectedCandidate.revision} · structural intake passed · visual judgment not assessed`
            : session
              ? "Concept slot ready. Import creates immutable, unreviewed candidates here."
              : "Begin a concept to create the first immutable candidate."}
        </p>
        {#if activeStage === 4 && worldTests.length > 0}
          <div class="candidate-strip" aria-label="Immutable World Test candidates">
            {#each worldTests as candidate}
              <button
                class:selected={selectedWorldTest?.id === candidate.id}
                onclick={() => loadWorldTest(candidate)}
                title={candidate.id}
              >
                <strong>r{candidate.revision}</strong>
                <span>local</span>
              </button>
            {/each}
          </div>
          <p class="turnaround-message" class:error={Boolean(worldTestError)} role={worldTestError ? "alert" : "status"}>
            {worldTestError || worldTestMessage}
          </p>
        {:else if activeStage === 3 && walkCycles.length > 0}
          <div class="candidate-strip" aria-label="Immutable Walk Cycle candidates">
            {#each walkCycles as candidate}
              <button
                class:selected={selectedWalkCycle?.id === candidate.id}
                onclick={() => loadWalkCycle(candidate)}
                title={candidate.id}
              >
                <strong>r{candidate.revision}</strong>
                <span>{candidate.provenance.source}</span>
              </button>
            {/each}
          </div>
          <p class="turnaround-message" class:error={Boolean(walkCycleError)} role={walkCycleError ? "alert" : "status"}>
            {walkCycleError || walkCycleMessage}
          </p>
        {:else if activeStage === 2 && turnarounds.length > 0}
          <div class="candidate-strip" aria-label="Immutable Turnaround candidates">
            {#each turnarounds as candidate}
              <button
                class:selected={selectedTurnaround?.id === candidate.id}
                onclick={() => loadTurnaround(candidate)}
                title={candidate.id}
              >
                <strong>r{candidate.revision}</strong>
                <span>{candidate.provenance.source}</span>
              </button>
            {/each}
          </div>
          <p class="turnaround-message" class:error={Boolean(turnaroundError)} role={turnaroundError ? "alert" : "status"}>
            {turnaroundError || turnaroundMessage}
          </p>
        {:else if candidates.length > 0}
          <div class="candidate-strip" aria-label="Immutable Concept candidates">
            {#each candidates as candidate}
              <button
                class:selected={selectedCandidate?.id === candidate.id}
                onclick={() => loadCandidate(candidate)}
                title={candidate.id}
              >
                <strong>r{candidate.revision}</strong>
                <span>{candidate.provenance.source}</span>
              </button>
            {/each}
          </div>
        {/if}
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
        {#if activeStage === 4 && selectedWorldPreviewUrl}
          <div class="world-swatch pinned">
            <img
              alt={`${selectedScene} ${selectedTheme} pinned reference thumbnail`}
              src={selectedWorldPreviewUrl}
            />
            <small>{selectedScene} · {selectedTheme}</small>
          </div>
        {:else}
          <div class="world-swatch" class:dusk={selectedTheme === "Dusk"} class:winter={selectedTheme === "Winter"}>
            <span class="terrain-grain one"></span>
            <span class="terrain-grain two"></span>
            <span class="terrain-grain three"></span>
            <div class="world-actor">
              <span></span>
            </div>
            <small>{selectedScene} · {selectedTheme}</small>
          </div>
        {/if}
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

      <section class="validation-card" aria-label="Structural validation report">
        <div class="validation-heading">
          <div>
            <p class="eyebrow">Local evidence</p>
            <strong>Contract validation</strong>
          </div>
          {#if activeStage === 4 && selectedWorldTest}
            <button
              aria-label="Run World Test ground validation again"
              disabled={validatingWorldTest}
              onclick={() => selectedWorldTest && validateWorldTest(selectedWorldTest)}
            >
              {validatingWorldTest ? "Running…" : "Re-run"}
            </button>
          {:else if activeStage === 3 && selectedWalkCycle}
            <button
              aria-label="Run Walk Cycle structural validation again"
              disabled={validatingWalkCycle}
              onclick={() => selectedWalkCycle && validateWalkCycle(selectedWalkCycle)}
            >
              {validatingWalkCycle ? "Running…" : "Re-run"}
            </button>
          {:else if activeStage === 2 && selectedTurnaround}
            <button
              aria-label="Run Turnaround structural validation again"
              disabled={validatingTurnaround}
              onclick={() => selectedTurnaround && validateTurnaround(selectedTurnaround)}
            >
              {validatingTurnaround ? "Running…" : "Re-run"}
            </button>
          {:else if selectedCandidate}
            <button
              aria-label="Run structural validation again"
              disabled={validating}
              onclick={() => selectedCandidate && validateCandidate(selectedCandidate)}
            >
              {validating ? "Running…" : "Re-run"}
            </button>
          {/if}
        </div>

        {#if activeStage === 4}
          {#if !selectedWorldTest}
            <p class="validation-empty">Prepare a World Test to measure all sixteen walk frames against sixteen pinned grounds.</p>
          {:else if validatingWorldTest && !worldTestValidation}
            <p class="validation-empty">Measuring 256 frame-to-ground luma comparisons locally…</p>
          {:else if worldTestError}
            <p class="validation-error" role="alert">{worldTestError}</p>
          {:else if worldTestValidation}
            <div class="validation-summary" aria-label="World Test validation totals">
              <span class="pass">{worldTestValidation.summary.pass} pass</span>
              <span class:quiet={worldTestValidation.summary.fail === 0} class="fail">
                {worldTestValidation.summary.fail} fail
              </span>
              <span class="not-assessed">0 not assessed</span>
            </div>
            <div class="validation-results">
              <div
                class:pass={selectedWorldGroundSummary.fail === 0}
                class:fail={selectedWorldGroundSummary.fail > 0}
                class="validation-result"
              >
                <span class="validation-mark" aria-label={selectedWorldGroundSummary.fail === 0 ? "pass" : "fail"}>
                  {selectedWorldGroundSummary.fail === 0 ? "✓" : "!"}
                </span>
                <div>
                  <strong>{selectedScene} · {selectedTheme}</strong>
                  <span>
                    {selectedWorldGroundSummary.pass} pass ·
                    {selectedWorldGroundSummary.fail} fail across 16 frames
                  </span>
                </div>
              </div>
              <div class="validation-result pass">
                <span class="validation-mark" aria-label="pass">✓</span>
                <div>
                  <strong>Pinned references</strong>
                  <span>4 scenes · 4 themes · SHA-256 verified</span>
                </div>
              </div>
            </div>
            <div class="visual-judgment">
              <span>Final art</span>
              <strong>Not assessed — user only</strong>
            </div>
          {/if}
        {:else if activeStage === 3}
          {#if !selectedWalkCycle}
            <p class="validation-empty">Create an immutable Walk Cycle to measure all sixteen frames.</p>
          {:else if validatingWalkCycle && !walkCycleValidation}
            <p class="validation-empty">Measuring sixteen immutable frame PNGs locally…</p>
          {:else if walkCycleError}
            <p class="validation-error" role="alert">{walkCycleError}</p>
          {:else if walkCycleValidation}
            <div class="validation-summary" aria-label="Walk Cycle validation totals">
              <span class="pass">{walkCycleValidation.summary.pass} pass</span>
              <span class:quiet={walkCycleValidation.summary.fail === 0} class="fail">
                {walkCycleValidation.summary.fail} fail
              </span>
              <span class="not-assessed">
                {walkCycleValidation.summary.notAssessed} not assessed
              </span>
            </div>
            <div class="validation-results">
              {#each TURNAROUND_DIRECTIONS as direction}
                {@const summary = walkDirectionSummary(direction)}
                <div
                  class:pass={summary.fail === 0}
                  class:fail={summary.fail > 0}
                  class="validation-result"
                >
                  <span class="validation-mark" aria-label={summary.fail === 0 ? "pass" : "fail"}>
                    {summary.fail === 0 ? "✓" : "!"}
                  </span>
                  <div>
                    <strong>{direction} · 4 frames</strong>
                    <span>
                      {summary.pass} pass · {summary.fail} fail ·
                      {summary.notAssessed} not assessed
                    </span>
                  </div>
                </div>
              {/each}
            </div>
            <div class="visual-judgment">
              <span>Motion and readability</span>
              <strong>Not assessed — user only</strong>
            </div>
          {/if}
        {:else if activeStage === 2}
          {#if !selectedTurnaround}
            <p class="validation-empty">Create an immutable Turnaround to measure all four views.</p>
          {:else if validatingTurnaround && !turnaroundValidation}
            <p class="validation-empty">Measuring four immutable direction PNGs locally…</p>
          {:else if turnaroundError}
            <p class="validation-error" role="alert">{turnaroundError}</p>
          {:else if turnaroundValidation}
            <div class="validation-summary" aria-label="Turnaround validation totals">
              <span class="pass">{turnaroundValidation.summary.pass} pass</span>
              <span class:quiet={turnaroundValidation.summary.fail === 0} class="fail">
                {turnaroundValidation.summary.fail} fail
              </span>
              <span class="not-assessed">
                {turnaroundValidation.summary.notAssessed} not assessed
              </span>
            </div>
            <div class="validation-results">
              {#each turnaroundValidation.directions as direction}
                <div
                  class:pass={direction.report.summary.fail === 0}
                  class:fail={direction.report.summary.fail > 0}
                  class="validation-result"
                >
                  <span class="validation-mark" aria-label={direction.report.summary.fail === 0 ? "pass" : "fail"}>
                    {direction.report.summary.fail === 0 ? "✓" : "!"}
                  </span>
                  <div>
                    <strong>{direction.direction}</strong>
                    <span>
                      {direction.report.summary.pass} pass · {direction.report.summary.fail} fail ·
                      {direction.report.summary.notAssessed} not assessed
                    </span>
                  </div>
                </div>
              {/each}
            </div>
            <div class="visual-judgment">
              <span>Identity consistency</span>
              <strong>Not assessed — user only</strong>
            </div>
          {/if}
        {:else if !selectedCandidate}
          <p class="validation-empty">Select or import a candidate to measure its structure.</p>
        {:else if validating && !validationReport}
          <p class="validation-empty">Measuring immutable PNG pixels locally…</p>
        {:else if validationError}
          <p class="validation-error" role="alert">{validationError}</p>
        {:else if validationReport}
          <div class="validation-summary" aria-label="Validation totals">
            <span class="pass">{validationReport.summary.pass} pass</span>
            <span class:quiet={validationReport.summary.fail === 0} class="fail">
              {validationReport.summary.fail} fail
            </span>
            <span class="not-assessed">
              {validationReport.summary.notAssessed} not assessed
            </span>
          </div>
          <div class="validation-results">
            {#each validationReport.results as result}
              <div class:pass={result.status === "pass"} class:fail={result.status === "fail"} class:not-assessed={result.status === "not_assessed"} class="validation-result">
                <span class="validation-mark" aria-label={result.status}>
                  {result.status === "pass" ? "✓" : result.status === "fail" ? "!" : "—"}
                </span>
                <div>
                  <strong>{validationLabels[result.id]}</strong>
                  <span>{result.observed ?? "Pinned ground reference unavailable"}</span>
                </div>
              </div>
            {/each}
          </div>
          <div class="visual-judgment">
            <span>Visual judgment</span>
            <strong>Not assessed — user only</strong>
          </div>
        {/if}
      </section>

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
