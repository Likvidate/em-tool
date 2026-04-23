export interface OneOnOne {
  id: number;
  reportId: number;
  occurredAt: number;       // unix seconds
  agendaMd: string | null;
  notesMd: string | null;
  createdAt: number;
}

export interface NewOneOnOneInput {
  reportId: number;
  occurredAt: number;
  agendaMd?: string | null;
  notesMd?: string | null;
}

export interface UpdateOneOnOneInput {
  id: number;
  occurredAt?: number;
  agendaMd?: string | null;
  notesMd?: string | null;
}
