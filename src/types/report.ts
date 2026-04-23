export interface Report {
  id: number;
  name: string;
  role: string | null;
  startDate: string | null;            // ISO date YYYY-MM-DD
  oneOnOneCadenceDays: number;
  notes: string | null;
  active: boolean;
  createdAt: number;                   // unix seconds
}

export interface NewReportInput {
  name: string;
  role?: string | null;
  startDate?: string | null;
  oneOnOneCadenceDays: number;
  notes?: string | null;
}

export interface UpdateReportInput {
  id: number;
  name?: string;
  role?: string | null;
  startDate?: string | null;
  oneOnOneCadenceDays?: number;
  notes?: string | null;
  active?: boolean;
}
