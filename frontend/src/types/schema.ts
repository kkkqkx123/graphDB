export interface Space {
  id: number;
  name: string;
  vid_type: string;
}

export interface SpaceDetail {
  id: number;
  name: string;
  vid_type: string;
  partition_num: number;
  replica_factor: number;
  comment?: string;
  created_at: number;
  statistics: SpaceStatistics;
}

export interface SpaceStatistics {
  vertex_count?: number;
  edge_count?: number;
}

export interface PropertyDef {
  name: string;
  data_type: string;
  nullable: boolean;
  default_value?: string;
  comment?: string;
}

export interface Tag {
  id: number;
  name: string;
  properties: PropertyDef[];
  created_at: number;
}

export interface TagDetail {
  id: number;
  name: string;
  properties: PropertyDef[];
  indexes: IndexInfo[];
  created_at: number;
}

export interface EdgeType {
  id: number;
  name: string;
  properties: PropertyDef[];
  created_at: number;
}

export interface EdgeTypeDetail {
  id: number;
  name: string;
  properties: PropertyDef[];
  indexes: IndexInfo[];
  created_at: number;
}

export interface IndexInfo {
  id: number;
  name: string;
  index_type: string;
  entity_type: string;
  entity_name: string;
  fields: string[];
  comment?: string;
  created_at: number;
}

export interface CreateTagParams {
  name: string;
  properties: PropertyDef[];
}

export interface CreateEdgeTypeParams {
  name: string;
  properties: PropertyDef[];
}

export interface CreateIndexParams {
  name: string;
  index_type: string;
  entity_type: string;
  entity_name: string;
  fields: string[];
  comment?: string;
}