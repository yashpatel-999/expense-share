import { Injectable } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';
import { Observable, throwError } from 'rxjs';

export interface ApiError {
  code?: string;
  error?: string;
  message: string;
  statusCode?: number;
}

@Injectable({
  providedIn: 'root'
})
export class ErrorHandlerService {

  constructor() { }

  handleError(error: HttpErrorResponse): Observable<never> {
    let errorMessage: ApiError;

    if (error.error instanceof ErrorEvent) {
      // Client-side error
      errorMessage = {
        message: `Client Error: ${error.error.message}`,
        statusCode: 0
      };
    } else {
      // Server-side error
      if (error.error && typeof error.error === 'object') {
        // Backend returned structured error
        errorMessage = {
          code: error.error.code || 'UNKNOWN_ERROR',
          error: error.error.error || 'Server Error',
          message: error.error.message || `HTTP ${error.status}: ${error.statusText}`,
          statusCode: error.status
        };
      } else {
        // Generic HTTP error
        errorMessage = {
          message: `HTTP ${error.status}: ${error.statusText || 'Unknown Error'}`,
          statusCode: error.status
        };
      }
    }

    console.error('API Error:', errorMessage);
    return throwError(() => errorMessage);
  }

  getUserFriendlyMessage(error: ApiError): string {
    // Map backend error codes to user-friendly messages
    switch (error.code) {
      case 'AUTH_INVALID_CREDENTIALS':
        return 'Invalid email or password. Please try again.';
      case 'AUTH_MISSING_JWT_SECRET':
        return 'Authentication configuration error. Please contact support.';
      case 'AUTH_INSUFFICIENT_PERMISSIONS':
        return 'You do not have permission to perform this action.';
      case 'USER_EMAIL_ALREADY_EXISTS':
        return 'An account with this email already exists.';
      case 'USER_USERNAME_ALREADY_EXISTS':
        return 'This username is already taken.';
      case 'USER_NOT_FOUND':
        return 'User not found.';
      case 'VALIDATION_REQUIRED_FIELD':
        return error.message || 'Please fill in all required fields.';
      case 'VALIDATION_INVALID_FORMAT':
        return error.message || 'Please check your input format.';
      case 'DATABASE_QUERY_FAILED':
        return 'Database error. Please try again later.';
      case 'GROUP_NOT_FOUND':
        return 'Group not found.';
      case 'GROUP_NOT_MEMBER':
        return 'You are not a member of this group.';
      case 'EXPENSE_NOT_FOUND':
        return 'Expense not found.';
      default:
        if (error.statusCode === 401) {
          return 'Your session has expired. Please log in again.';
        } else if (error.statusCode === 403) {
          return 'You do not have permission to access this resource.';
        } else if (error.statusCode === 404) {
          return 'The requested resource was not found.';
        } else if (error.statusCode === 500) {
          return 'Server error. Please try again later.';
        } else if (error.statusCode === 0) {
          return 'Cannot connect to server. Please check your connection.';
        }
        return error.message || 'An unexpected error occurred.';
    }
  }
}