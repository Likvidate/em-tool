export interface PerformanceReview {
  id: number;
  reportId: number;
  period: string;            // e.g. "Q1 2026"
  rating: string | null;
  strengthsMd: string | null;
  devAreasMd: string | null;
  goalsMd: string | null;
  occurredAt: number;
  createdAt: number;
}

export interface NewPerformanceReviewInput {
  reportId: number;
  period: string;
  rating?: string | null;
  strengthsMd?: string | null;
  devAreasMd?: string | null;
  goalsMd?: string | null;
  occurredAt: number;
}
