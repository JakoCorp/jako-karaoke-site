import type { TagAssignment } from "./pickers";

export async function resolveTagAssignments<K extends string>(
  assignments: TagAssignment<K>[],
  createTag: (name: string) => Promise<string>,
): Promise<Array<{ tag_id: string; kind: K }>> {
  return Promise.all(
    assignments.map(async (assignment) => {
      if (assignment.type === "pending") {
        const tagId = await createTag(assignment.pendingName);
        return { tag_id: tagId, kind: assignment.kind };
      }
      return { tag_id: assignment.tagId, kind: assignment.kind };
    }),
  );
}
