import { post, get } from '@/utils/http';

export interface LoginParams {
  username: string;
  password: string;
}

export interface LoginResponse {
  session_id: number;
  username: string;
  expires_at?: number;
}

export interface LogoutParams {
  session_id: number;
}

export interface HealthResponse {
  status: string;
  service: string;
  version: string;
}

export const connectionService = {
  login: async (params: LoginParams): Promise<LoginResponse> => {
    const response = await post('/v1/auth/login')(params) as LoginResponse;
    return response;
  },

  logout: async (sessionId: number): Promise<void> => {
    await post('/v1/auth/logout')({ session_id: sessionId });
  },

  health: async (): Promise<HealthResponse> => {
    const response = await get('/v1/health')() as HealthResponse;
    return response;
  },
};
