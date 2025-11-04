import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable, BehaviorSubject, throwError } from 'rxjs';
import { tap, catchError } from 'rxjs/operators';
import { User, LoginRequest, LoginResponse, CreateUserRequest } from '../models/user.interface';
import { Group, CreateGroupRequest, Balance, CreateExpense, CreatePayment } from '../models/group.interface';
import { environment } from '../../environments/environment';

@Injectable({
  providedIn: 'root'
})
export class ApiService {
  getGroupExpenses(groupId: string): Observable<any[]> {
    return this.http.get<any[]>(`${this.baseUrl}/groups/${groupId}/expenses`, {
      headers: this.getAuthHeaders()
    }).pipe(
      catchError(this.handleError)
    );
  }
  private readonly baseUrl = environment.apiUrl;
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
    console.log('Making login request to:', `${this.baseUrl}/auth/login`);
    console.log('Request data:', request);
    
    return this.http.post<LoginResponse>(`${this.baseUrl}/auth/login`, request).pipe(
      tap(response => {
        console.log('Login response received:', response);
        this.setToken(response.token);
        this.setUser(response.user);
        this.currentUserSubject.next(response.user);
      }),
      catchError((error) => {
        console.error('Login error in service:', error);
        return throwError(() => error);
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

  // Simple error handler
  private handleError(error: any): Observable<never> {
    console.error('API Error:', error);
    return throwError(() => error);
  }

  // Admin API methods
  getAllUsers(): Observable<User[]> {
    return this.http.get<User[]>(`${this.baseUrl}/users`, {
      headers: this.getAuthHeaders()
    }).pipe(
      catchError(this.handleError)
    );
  }

  createUser(request: CreateUserRequest): Observable<User> {
    return this.http.post<User>(`${this.baseUrl}/users`, request, {
      headers: this.getAuthHeaders()
    }).pipe(
      catchError(this.handleError)
    );
  }

  createGroup(request: CreateGroupRequest): Observable<Group> {
    return this.http.post<Group>(`${this.baseUrl}/groups`, request, {
      headers: this.getAuthHeaders()
    }).pipe(
      catchError(this.handleError)
    );
  }

  // User API methods
  getUserGroups(): Observable<Group[]> {
    return this.http.get<Group[]>(`${this.baseUrl}/groups`, {
      headers: this.getAuthHeaders()
    }).pipe(
      catchError(this.handleError)
    );
  }

  getGroupBalances(groupId: string): Observable<Balance[]> {
    return this.http.get<Balance[]>(`${this.baseUrl}/groups/${groupId}/balances`, {
      headers: this.getAuthHeaders()
    }).pipe(
      catchError(this.handleError)
    );
  }

  addExpense(groupId: string, expense: CreateExpense): Observable<any> {
    return this.http.post(`${this.baseUrl}/groups/${groupId}/expenses`, expense, {
      headers: this.getAuthHeaders()
    }).pipe(
      catchError(this.handleError)
    );
  }

  recordPayment(groupId: string, payment: CreatePayment): Observable<any> {
    return this.http.post(`${this.baseUrl}/groups/${groupId}/payments`, payment, {
      headers: this.getAuthHeaders()
    }).pipe(
      catchError(this.handleError)
    );
  }
}
