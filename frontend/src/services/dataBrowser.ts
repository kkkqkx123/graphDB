import { get } from '@/utils/http';
import type {
  VertexListResponse,
  EdgeListResponse,
  FilterGroup,
  Statistics,
} from '@/types/dataBrowser';

export interface DataBrowserService {
  getVertices: (
    space: string,
    tag: string,
    page: number,
    pageSize: number,
    sort: { field: string; order: 'asc' | 'desc' },
    filters: FilterGroup
  ) => Promise<VertexListResponse>;

  getEdges: (
    space: string,
    type: string,
    page: number,
    pageSize: number,
    sort: { field: string; order: 'asc' | 'desc' },
    filters: FilterGroup
  ) => Promise<EdgeListResponse>;

  getStatistics: (space: string) => Promise<Statistics>;
}

export const dataBrowserService: DataBrowserService = {
  getVertices: async (space, tag, page, pageSize, sort, filters) => {
    const params: Record<string, string | number> = {
      space,
      tag,
      page,
      pageSize,
      sortField: sort.field,
      sortOrder: sort.order,
    };

    if (filters && filters.conditions.length > 0) {
      params.filters = JSON.stringify(filters);
    }

    const response = await get('/api/data/vertices')(params) as VertexListResponse;
    return response;
  },

  getEdges: async (space, type, page, pageSize, sort, filters) => {
    const params: Record<string, string | number> = {
      space,
      type,
      page,
      pageSize,
      sortField: sort.field,
      sortOrder: sort.order,
    };

    if (filters && filters.conditions.length > 0) {
      params.filters = JSON.stringify(filters);
    }

    const response = await get('/api/data/edges')(params) as EdgeListResponse;
    return response;
  },

  getStatistics: async (space) => {
    const response = await get('/api/data/statistics')({ space }) as Statistics;
    return response;
  },
};
