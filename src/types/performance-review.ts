export interface PerformanceReview {
  id: number;
  reportId: number;
  period: string;
  rating: string | null;
  strengthsMd: string | null;
  devAreasMd: string | null;
  goalsMd: string | null;
  notesMd: string | null;
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
  notesMd?: string | null;
  occurredAt: number;
}

export interface UpdatePerformanceReviewInput {
  id: number;
  period?: string;
  rating?: string | null;
  strengthsMd?: string | null;
  devAreasMd?: string | null;
  goalsMd?: string | null;
  notesMd?: string | null;
  occurredAt?: number;
}
