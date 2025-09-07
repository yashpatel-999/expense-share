import { Component, OnInit } from '@angular/core';
import { Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import { ApiService } from '../../services/api.service';
import { Group } from '../../models/group.interface';
import { User } from '../../models/user.interface';

@Component({
  selector: 'app-user-dashboard',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './user-dashboard.component.html',
  styleUrl: './user-dashboard.component.css'
})
export class UserDashboardComponent implements OnInit {
  groups: Group[] = [];
  currentUser: User | null = null;
  isLoading = true;
  errorMessage = '';

  constructor(
    private apiService: ApiService,
    private router: Router
  ) {}

  ngOnInit() {
    this.currentUser = this.apiService.getCurrentUser();
    this.loadGroups();
  }

  loadGroups() {
    this.isLoading = true;
    this.errorMessage = '';

    this.apiService.getUserGroups().subscribe({
      next: (groups) => {
        this.groups = groups;
        this.isLoading = false;
      },
      error: (error) => {
        this.isLoading = false;
        this.errorMessage = 'Failed to load groups. Please try again.';
        console.error('Error loading groups:', error);
      }
    });
  }

  viewGroup(groupId: string) {
    this.router.navigate(['/group', groupId]);
  }

  formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  }

  refreshGroups() {
    this.loadGroups();
  }
}
