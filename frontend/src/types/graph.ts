export interface VertexDetail {
  vid: string | number;
  tags: Record<string, Record<string, unknown>>;
}

export interface EdgeDetail {
  src: string | number;
  dst: string | number;
  edge_type: string;
  rank: number;
  properties: Record<string, unknown>;
}

export interface NeighborParams {
  direction?: 'OUT' | 'IN' | 'BOTH';
  edge_type?: string;
}

export interface Neighbor {
  vid: string | number;
  edge_type: string;
  direction: 'OUT' | 'IN';
  rank: number;
}