export interface User {
  id: string;
  email: string;
  username: string;
  is_admin: boolean;
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface LoginResponse {
  message?: string;
  token: string;
  user: User;
}

export interface CreateUserRequest {
  email: string;
  username: string;
  password: string;
  is_admin?: boolean;
}
