import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { schemaService } from '@/services/schemaService';
import { queryService } from '@/services/query';
import type { Space, SpaceDetail, SpaceStatistics } from '@/types/schema';

export interface CreateSpaceParams {
  name: string;
  vidType: 'INT64' | 'FIXED_STRING(32)';
  partitionNum: number;
  replicaFactor: number;
}

export interface SchemaState {
  // Space list
  spaces: Space[];
  isLoadingSpaces: boolean;
  spacesError: string | null;

  // Current space
  currentSpace: string | null;

  // Space details cache
  spaceDetails: Record<string, SpaceDetail>;
  spaceStatistics: Record<string, SpaceStatistics>;

  // Actions
  fetchSpaces: () => Promise<void>;
  createSpace: (params: CreateSpaceParams) => Promise<void>;
  deleteSpace: (name: string) => Promise<void>;
  setCurrentSpace: (name: string | null) => void;
  fetchSpaceDetail: (name: string) => Promise<void>;
  fetchSpaceStatistics: (name: string) => Promise<void>;
  clearSpacesError: () => void;
}

export const useSchemaStore = create<SchemaState>()(
  persist(
    (set, get) => ({
      spaces: [],
      isLoadingSpaces: false,
      spacesError: null,
      currentSpace: null,
      spaceDetails: {},
      spaceStatistics: {},

      fetchSpaces: async () => {
        set({ isLoadingSpaces: true, spacesError: null });
        try {
          const spaces = await schemaService.spaces.list();
          set({ spaces, isLoadingSpaces: false });

          // If no current space is selected and spaces exist, select the first one
          const { currentSpace } = get();
          if (!currentSpace && spaces.length > 0) {
            set({ currentSpace: spaces[0].name });
          }
        } catch (err: unknown) {
          const errorMessage = err instanceof Error ? err.message : 'Failed to fetch spaces';
          set({ spacesError: errorMessage, isLoadingSpaces: false });
        }
      },

      createSpace: async (params: CreateSpaceParams) => {
        try {
          // Note: The backend API might need to be extended to support space creation
          // For now, we'll use the query service to execute CREATE SPACE command
          const vidTypeStr = params.vidType === 'FIXED_STRING(32)' ? 'FIXED_STRING(32)' : 'INT64';
          const query = `CREATE SPACE IF NOT EXISTS ${params.name} (vid_type = ${vidTypeStr}, partition_num = ${params.partitionNum}, replica_factor = ${params.replicaFactor})`;

          await queryService.execute({ query });

          // Refresh space list after creation
          await get().fetchSpaces();
        } catch (err: unknown) {
          const errorMessage = err instanceof Error ? err.message : 'Failed to create space';
          throw new Error(errorMessage);
        }
      },

      deleteSpace: async (name: string) => {
        try {
          // Note: The backend API might need to be extended to support space deletion
          // For now, we'll use the query service to execute DROP SPACE command
          const query = `DROP SPACE IF EXISTS ${name}`;

          await queryService.execute({ query });

          // Refresh space list after deletion
          await get().fetchSpaces();

          // If the deleted space was the current space, clear it
          const { currentSpace } = get();
          if (currentSpace === name) {
            const { spaces } = get();
            set({ currentSpace: spaces.length > 0 ? spaces[0].name : null });
          }
        } catch (err: unknown) {
          const errorMessage = err instanceof Error ? err.message : 'Failed to delete space';
          throw new Error(errorMessage);
        }
      },

      setCurrentSpace: (name: string | null) => {
        set({ currentSpace: name });
      },

      fetchSpaceDetail: async (name: string) => {
        try {
          const detail = await schemaService.spaces.getDetail(name);
          set((state) => ({
            spaceDetails: {
              ...state.spaceDetails,
              [name]: detail,
            },
          }));
        } catch (err: unknown) {
          const errorMessage = err instanceof Error ? err.message : 'Failed to fetch space detail';
          console.error('Fetch space detail error:', errorMessage);
        }
      },

      fetchSpaceStatistics: async (name: string) => {
        try {
          const statistics = await schemaService.spaces.getStatistics(name);
          set((state) => ({
            spaceStatistics: {
              ...state.spaceStatistics,
              [name]: statistics,
            },
          }));
        } catch (err: unknown) {
          const errorMessage = err instanceof Error ? err.message : 'Failed to fetch space statistics';
          console.error('Fetch space statistics error:', errorMessage);
        }
      },

      clearSpacesError: () => {
        set({ spacesError: null });
      },
    }),
    {
      name: 'schema-storage',
      partialize: (state) => ({ currentSpace: state.currentSpace }),
    }
  )
);
