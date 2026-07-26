import { createHash } from "node:crypto";
import { PNG } from "pngjs";
import {
  parseConceptCandidate,
  type ConceptCandidate,
} from "./candidate";
import { TILEFORGE_ACTOR_CONTRACT } from "./contract";
import {
  parseValidationReport,
  STRUCTURAL_VALIDATOR_ID,
  type ValidationReport,
  type ValidationRuleResult,
  type ValidationStatus,
} from "./validation";

function ruleResult(
  id: ValidationRuleResult["id"],
  status: ValidationStatus,
  expected: string,
  observed: string | null,
  message: string,
): ValidationRuleResult {
  return { id, status, expected, observed, message };
}

function pixelLabel(count: number): string {
  return `${count} pixel${count === 1 ? "" : "s"}`;
}

export function validateConceptCandidatePng(
  candidateInput: ConceptCandidate,
  pngBytes: Uint8Array,
): ValidationReport {
  const candidate = parseConceptCandidate(candidateInput);
  return validatePngStructuralEvidence(
    {
      artifactId: candidate.id,
      sha256: candidate.sha256,
      byteLength: candidate.byteLength,
      contractId: candidate.contractId,
    },
    pngBytes,
  );
}

interface StructuralArtifactIdentity {
  artifactId: string;
  sha256: string;
  byteLength: number;
  contractId: string;
}

export function validatePngStructuralEvidence(
  identity: StructuralArtifactIdentity,
  pngBytes: Uint8Array,
): ValidationReport {
  const sourceSha256 = createHash("sha256").update(pngBytes).digest("hex");
  if (
    pngBytes.byteLength !== identity.byteLength ||
    sourceSha256 !== identity.sha256
  ) {
    throw new Error("Artifact source bytes no longer match immutable provenance.");
  }

  let decoded: PNG;
  try {
    decoded = PNG.sync.read(Buffer.from(pngBytes), { checkCRC: true });
  } catch {
    throw new Error("Candidate source is not a valid PNG.");
  }

  const contract = TILEFORGE_ACTOR_CONTRACT;
  const visibleColors = new Set<string>();
  const edgeSides = new Set<string>();
  let edgePixelCount = 0;
  let semiTransparentPixelCount = 0;
  let visiblePixelCount = 0;
  let minY = decoded.height;
  let maxY = -1;
  let footAnchorContact = false;

  for (let y = 0; y < decoded.height; y += 1) {
    for (let x = 0; x < decoded.width; x += 1) {
      const index = (y * decoded.width + x) * 4;
      const red = decoded.data[index] ?? 0;
      const green = decoded.data[index + 1] ?? 0;
      const blue = decoded.data[index + 2] ?? 0;
      const alpha = decoded.data[index + 3] ?? 255;

      if (alpha > 0 && alpha < 255) {
        semiTransparentPixelCount += 1;
      }
      if (alpha === 0) {
        continue;
      }

      visiblePixelCount += 1;
      visibleColors.add(`${red},${green},${blue}`);
      minY = Math.min(minY, y);
      maxY = Math.max(maxY, y);
      if (x === contract.frame.footAnchor[0] && y === contract.frame.footAnchor[1]) {
        footAnchorContact = true;
      }
      if (
        x === 0 ||
        y === 0 ||
        x === decoded.width - 1 ||
        y === decoded.height - 1
      ) {
        edgePixelCount += 1;
        if (y === 0) edgeSides.add("top");
        if (x === decoded.width - 1) edgeSides.add("right");
        if (y === decoded.height - 1) edgeSides.add("bottom");
        if (x === 0) edgeSides.add("left");
      }
    }
  }

  const actorHeight = visiblePixelCount === 0 ? 0 : maxY - minY + 1;
  const dimensionsPass =
    decoded.width === contract.frame.width &&
    decoded.height === contract.frame.height;
  const hardAlphaPass = semiTransparentPixelCount === 0;
  const actorHeightPass =
    actorHeight >= contract.frame.actorHeightMin &&
    actorHeight <= contract.frame.actorHeightMax;
  const palettePass = visibleColors.size <= contract.art.paletteMaxColors;
  const edgePass = edgePixelCount === 0;
  const orderedEdgeSides = ["top", "right", "bottom", "left"].filter((side) =>
    edgeSides.has(side),
  );
  const edgeObserved = edgePass
    ? "No edge contact"
    : `${pixelLabel(edgePixelCount)} on ${orderedEdgeSides.join(", ")}`;

  const results: ValidationRuleResult[] = [
    ruleResult(
      "canvas_dimensions",
      dimensionsPass ? "pass" : "fail",
      `${contract.frame.width} x ${contract.frame.height} px`,
      `${decoded.width} x ${decoded.height} px`,
      dimensionsPass
        ? "Decoded canvas matches the contract."
        : "Decoded canvas dimensions do not match the contract.",
    ),
    ruleResult(
      "hard_alpha",
      hardAlphaPass ? "pass" : "fail",
      "Only alpha 0 or 255",
      `${pixelLabel(semiTransparentPixelCount)} with alpha from 1 to 254`,
      hardAlphaPass
        ? "All pixels use hard alpha."
        : "Semi-transparent pixels violate the hard-alpha contract.",
    ),
    ruleResult(
      "actor_height",
      actorHeightPass ? "pass" : "fail",
      `${contract.frame.actorHeightMin}-${contract.frame.actorHeightMax} px`,
      `${actorHeight} px`,
      actorHeightPass
        ? "Visible actor height is within the contract range."
        : "Visible actor height is outside the contract range.",
    ),
    ruleResult(
      "foot_anchor",
      footAnchorContact ? "pass" : "fail",
      `Visible pixel at (${contract.frame.footAnchor.join(", ")})`,
      footAnchorContact ? "Contact" : "No contact",
      footAnchorContact
        ? "The actor contacts the contract foot anchor."
        : "The contract foot anchor is transparent.",
    ),
    ruleResult(
      "palette_max_colors",
      palettePass ? "pass" : "fail",
      `${contract.art.paletteMaxColors} visible RGB colors or fewer`,
      `${visibleColors.size} visible RGB color${visibleColors.size === 1 ? "" : "s"}`,
      palettePass
        ? "Visible palette is within the contract maximum."
        : "Visible palette exceeds the contract maximum.",
    ),
    ruleResult(
      "ground_luma_separation",
      "not_assessed",
      `At least ${contract.art.minimumGroundLumaDistance} luma from pinned ground`,
      null,
      "A pinned ground reference is required before this rule can be measured.",
    ),
    ruleResult(
      "frame_edge_clipping",
      edgePass ? "pass" : "fail",
      "No visible pixels on the frame edge",
      edgeObserved,
      edgePass
        ? "No visible pixel touches the frame edge."
        : "Visible edge contact indicates possible clipping.",
    ),
  ];

  const summary = results.reduce(
    (counts, result) => {
      if (result.status === "not_assessed") {
        counts.notAssessed += 1;
      } else {
        counts[result.status] += 1;
      }
      return counts;
    },
    { pass: 0, fail: 0, notAssessed: 0 },
  );

  return parseValidationReport({
    schemaVersion: 1,
    validatorId: STRUCTURAL_VALIDATOR_ID,
    candidateId: identity.artifactId,
    candidateSha256: identity.sha256,
    contractId: identity.contractId,
    results,
    summary,
    visualJudgment: {
      status: "not_assessed",
      authority: "user",
      message: "Only the user can make the visual-acceptance decision.",
    },
  });
}
