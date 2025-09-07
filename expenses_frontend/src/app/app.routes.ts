import { Routes } from '@angular/router';
import { inject } from '@angular/core';
import { ApiService } from './services/api.service';
import { Router } from '@angular/router';

// Auth Guard
function authGuard() {
  const apiService = inject(ApiService);
  const router = inject(Router);
  
  if (apiService.isLoggedIn()) {
    return true;
  } else {
    router.navigate(['/login']);
    return false;
  }
}

// Admin Guard
function adminGuard() {
  const apiService = inject(ApiService);
  const router = inject(Router);
  
  if (apiService.isLoggedIn() && apiService.isAdmin()) {
    return true;
  } else {
    router.navigate(['/dashboard']);
    return false;
  }
}

export const routes: Routes = [
  { path: '', redirectTo: '/login', pathMatch: 'full' },
  { 
    path: 'login', 
    loadComponent: () => import('./components/login/login.component').then(m => m.LoginComponent)
  },
  { 
    path: 'admin', 
    canActivate: [adminGuard],
    loadComponent: () => import('./components/admin-dashboard/admin-dashboard.component').then(m => m.AdminDashboardComponent)
  },
  { 
    path: 'dashboard', 
    canActivate: [authGuard],
    loadComponent: () => import('./components/user-dashboard/user-dashboard.component').then(m => m.UserDashboardComponent)
  },
  { 
    path: 'group/:id', 
    canActivate: [authGuard],
    loadComponent: () => import('./components/group-detail/group-detail.component').then(m => m.GroupDetailComponent)
  },
  { path: '**', redirectTo: '/login' }
];
