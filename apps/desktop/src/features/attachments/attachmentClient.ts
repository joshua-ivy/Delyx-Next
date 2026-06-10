import { invoke } from "@tauri-apps/api/core";
import type {
  AttachmentParseResult,
  AttachmentProposal,
  AttachmentProposeRequest,
  AttachmentRecord,
  AttachmentSnapshotView,
  ContextPack,
} from "./attachmentTypes";

/**
 * Attachment proposal bridge. Proposing only previews + classifies a source; it
 * never reads file contents (parsing lands in a later PR). Requires the desktop
 * runtime — callers handle rejection in the web preview.
 */
export async function proposeAttachment(request: AttachmentProposeRequest): Promise<AttachmentProposal> {
  return invoke<AttachmentProposal>("attachment_propose", { request });
}

export async function loadAttachmentSnapshot(
  projectId: string,
  threadId?: string,
): Promise<AttachmentSnapshotView> {
  return invoke<AttachmentSnapshotView>("attachment_snapshot", { projectId, threadId });
}

/** Accept a proposal into a durable record. Risky proposals need an approvalId. */
export async function approveAttachment(proposalId: string, approvalId?: string): Promise<AttachmentRecord> {
  return invoke<AttachmentRecord>("attachment_approve", { request: { proposalId, approvalId } });
}

export async function denyAttachment(proposalId: string): Promise<AttachmentProposal> {
  return invoke<AttachmentProposal>("attachment_deny", { request: { proposalId } });
}

export async function expireAttachment(proposalId: string): Promise<AttachmentProposal> {
  return invoke<AttachmentProposal>("attachment_expire", { request: { proposalId } });
}

/** Parse an accepted text/code/markdown attachment into chunks. */
export async function parseAttachment(attachmentId: string, content?: string): Promise<AttachmentParseResult> {
  return invoke<AttachmentParseResult>("attachment_parse", { request: { attachmentId, content } });
}

/** Build the thread's context pack from parsed attachment chunks. */
export async function createContextPack(
  projectId: string,
  threadId: string,
  options?: { budgetTokens?: number; pinnedLocators?: string[] },
): Promise<ContextPack> {
  return invoke<ContextPack>("context_pack_create", {
    request: { projectId, threadId, budgetTokens: options?.budgetTokens, pinnedLocators: options?.pinnedLocators },
  });
}
