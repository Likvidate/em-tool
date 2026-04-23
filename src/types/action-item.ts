export type ActionItemOwner = "me" | "them";

export interface ActionItem {
  id: number;
  oneOnOneId: number | null;
  reportId: number;
  text: string;
  owner: ActionItemOwner;
  dueDate: string | null;    // YYYY-MM-DD
  completedAt: number | null;
  createdAt: number;
}

export interface NewActionItemInput {
  oneOnOneId?: number | null;
  reportId: number;
  text: string;
  owner: ActionItemOwner;
  dueDate?: string | null;
}
