import { Component } from '@angular/core';
import { Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ApiService } from '../../services/api.service';
import { LoginRequest } from '../../models/user.interface';

@Component({
  selector: 'app-login',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './login.component.html',
  styleUrl: './login.component.css'
})
export class LoginComponent {
  loginData: LoginRequest = {
    email: '',
    password: ''
  };
  
  isLoading = false;
  errorMessage = '';

  constructor(
    private apiService: ApiService,
    private router: Router
  ) {
    // Redirect if already logged in
    if (this.apiService.isLoggedIn()) {
      this.redirectBasedOnRole();
    }
  }

  onSubmit() {
    if (!this.loginData.email || !this.loginData.password) {
      this.errorMessage = 'Please fill in all fields';
      return;
    }

    console.log('Login form submitted with:', this.loginData);
    this.isLoading = true;
    this.errorMessage = '';

    this.apiService.login(this.loginData).subscribe({
      next: (response) => {
        console.log('Login successful in component:', response);
        this.isLoading = false;
        this.redirectBasedOnRole();
      },
      error: (error: any) => {
        console.error('Login error in component:', error);
        this.isLoading = false;
        this.errorMessage = error.message || error.error?.message || 'Login failed. Please try again.';
      }
    });
  }

  private redirectBasedOnRole() {
    console.log('Checking user role for redirect...');
    console.log('Current user:', this.apiService.getCurrentUser());
    console.log('Is admin?', this.apiService.isAdmin());
    
    if (this.apiService.isAdmin()) {
      console.log('Redirecting to admin dashboard');
      this.router.navigate(['/admin']);
    } else {
      console.log('Redirecting to user dashboard');
      this.router.navigate(['/dashboard']);
    }
  }
}
