import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable, BehaviorSubject } from 'rxjs';
import { tap } from 'rxjs/operators';
import { User, LoginRequest, LoginResponse, CreateUserRequest } from '../models/user.interface';
import { Group, CreateGroupRequest, Balance, CreateExpense, CreatePayment } from '../models/group.interface';

@Injectable({
  providedIn: 'root'
})
export class ApiService {
  getGroupExpenses(groupId: string): Observable<any[]> {
    return this.http.get<any[]>(`${this.baseUrl}/groups/${groupId}/expenses`, {
      headers: this.getAuthHeaders()
    });
  }
  private readonly baseUrl = 'http://localhost:8080';
  private currentUserSubject = new BehaviorSubject<User | null>(null);
  public currentUser$ = this.currentUserSubject.asObservable();

  constructor(private http: HttpClient) {
    // Load user from localStorage on service initialization
    const token = this.getToken();
    const userStr = localStorage.getItem('user');
    if (token && userStr) {
      try {
        const user = JSON.parse(userStr);
        this.currentUserSubject.next(user);
      } catch (error) {
        console.error('Error parsing stored user:', error);
        this.clearAuth();
      }
    }
  }

  // Auth methods
  login(request: LoginRequest): Observable<LoginResponse> {
    return this.http.post<LoginResponse>(`${this.baseUrl}/login`, request).pipe(
      tap(response => {
        this.setToken(response.token);
        this.setUser(response.user);
        this.currentUserSubject.next(response.user);
      })
    );
  }

  logout(): void {
    this.clearAuth();
    this.currentUserSubject.next(null);
  }

  isLoggedIn(): boolean {
    return !!this.getToken();
  }

  getCurrentUser(): User | null {
    return this.currentUserSubject.value;
  }

  isAdmin(): boolean {
    const user = this.getCurrentUser();
    return user?.is_admin === true;
  }

  // Token management
  private getToken(): string | null {
    return localStorage.getItem('token');
  }

  private setToken(token: string): void {
    localStorage.setItem('token', token);
  }

  private setUser(user: User): void {
    localStorage.setItem('user', JSON.stringify(user));
  }

  private clearAuth(): void {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
  }

  // HTTP headers with auth
  private getAuthHeaders(): HttpHeaders {
    const token = this.getToken();
    return new HttpHeaders({
      'Content-Type': 'application/json',
      ...(token && { Authorization: `Bearer ${token}` })
    });
  }

  // Admin API methods
  getAllUsers(): Observable<User[]> {
    return this.http.get<User[]>(`${this.baseUrl}/admin/users`, {
      headers: this.getAuthHeaders()
    });
  }

  createUser(request: CreateUserRequest): Observable<User> {
    return this.http.post<User>(`${this.baseUrl}/admin/users`, request, {
      headers: this.getAuthHeaders()
    });
  }

  createGroup(request: CreateGroupRequest): Observable<Group> {
    return this.http.post<Group>(`${this.baseUrl}/admin/groups`, request, {
      headers: this.getAuthHeaders()
    });
  }

  // User API methods
  getUserGroups(): Observable<Group[]> {
    return this.http.get<Group[]>(`${this.baseUrl}/groups`, {
      headers: this.getAuthHeaders()
    });
  }

  getGroupBalances(groupId: string): Observable<Balance[]> {
    return this.http.get<Balance[]>(`${this.baseUrl}/groups/${groupId}/balances`, {
      headers: this.getAuthHeaders()
    });
  }

  addExpense(groupId: string, expense: CreateExpense): Observable<any> {
    return this.http.post(`${this.baseUrl}/groups/${groupId}/expenses`, expense, {
      headers: this.getAuthHeaders()
    });
  }

  recordPayment(groupId: string, payment: CreatePayment): Observable<any> {
    return this.http.post(`${this.baseUrl}/groups/${groupId}/payments`, payment, {
      headers: this.getAuthHeaders()
    });
  }
}
