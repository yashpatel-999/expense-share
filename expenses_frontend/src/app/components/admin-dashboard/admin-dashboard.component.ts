import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ApiService } from '../../services/api.service';
import { User, CreateUserRequest } from '../../models/user.interface';
import { CreateGroupRequest } from '../../models/group.interface';

@Component({
  selector: 'app-admin-dashboard',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './admin-dashboard.component.html',
  styleUrl: './admin-dashboard.component.css'
})
export class AdminDashboardComponent implements OnInit {
  users: User[] = [];
  selectedUsers: string[] = [];
  
  newUser: CreateUserRequest = {
    email: '',
    username: '',
    password: '',
    is_admin: false
  };
  
  newGroup: CreateGroupRequest = {
    name: '',
    user_ids: []
  };
  
  isLoadingUsers = false;
  isCreatingUser = false;
  isCreatingGroup = false;
  
  userError = '';
  groupError = '';
  userSuccess = '';
  groupSuccess = '';

  constructor(private apiService: ApiService) {}

  ngOnInit() {
    this.loadUsers();
  }

  loadUsers() {
    this.isLoadingUsers = true;
    this.apiService.getAllUsers().subscribe({
      next: (users) => {
        this.users = users;
        this.isLoadingUsers = false;
      },
      error: (error) => {
        console.error('Error loading users:', error);
        this.isLoadingUsers = false;
        alert('Error loading users. Please try again.');
      }
    });
  }

  createUser() {
    if (!this.newUser.email || !this.newUser.username || !this.newUser.password) {
      this.userError = 'Please fill in all fields';
      return;
    }

    this.isCreatingUser = true;
    this.userError = '';
    this.userSuccess = '';

    this.apiService.createUser(this.newUser).subscribe({
      next: (user) => {
        this.isCreatingUser = false;
        this.userSuccess = `User ${user.username} created successfully!`;
        this.newUser = { email: '', username: '', password: '', is_admin: false };
        this.loadUsers(); // Refresh the list
      },
      error: (error) => {
        this.isCreatingUser = false;
        this.userError = error.error?.message || 'Failed to create user. Please try again.';
        console.error('Create user error:', error);
      }
    });
  }

  toggleUserSelection(userId: string) {
    const index = this.selectedUsers.indexOf(userId);
    if (index === -1) {
      this.selectedUsers.push(userId);
    } else {
      this.selectedUsers.splice(index, 1);
    }
  }

  createGroup() {
    if (!this.newGroup.name) {
      this.groupError = 'Please enter a group name';
      return;
    }

    if (this.selectedUsers.length === 0) {
      this.groupError = 'Please select at least one user for the group';
      return;
    }

    this.isCreatingGroup = true;
    this.groupError = '';
    this.groupSuccess = '';

    const groupRequest: CreateGroupRequest = {
      name: this.newGroup.name,
      user_ids: [...this.selectedUsers]
    };

    this.apiService.createGroup(groupRequest).subscribe({
      next: (group) => {
        this.isCreatingGroup = false;
        this.groupSuccess = `Group "${group.name}" created successfully!`;
        this.newGroup.name = '';
        this.selectedUsers = [];
      },
      error: (error) => {
        this.isCreatingGroup = false;
        this.groupError = error.error?.message || 'Failed to create group. Please try again.';
        console.error('Create group error:', error);
      }
    });
  }

  isUserSelected(userId: string): boolean {
    return this.selectedUsers.includes(userId);
  }

  getUserById(userId: string): User | undefined {
    return this.users.find(user => user.id === userId);
  }
}
