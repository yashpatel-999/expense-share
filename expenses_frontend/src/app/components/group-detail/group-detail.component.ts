import { Component, OnInit } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ApiService } from '../../services/api.service';
import { Balance, CreateExpense, CreatePayment } from '../../models/group.interface';
import { User } from '../../models/user.interface';

@Component({
  selector: 'app-group-detail',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './group-detail.component.html',
  styleUrl: './group-detail.component.css'
})
export class GroupDetailComponent implements OnInit {
  expenses: any[] = [];
  
  // Epsilon for floating-point comparisons (0.01 = 1 cent precision)
  // This prevents issues with tiny precision errors like 0.00001 being treated as non-zero
  private readonly EPSILON = 0.01;
  
  loadExpenses() {
    this.apiService.getGroupExpenses(this.groupId).subscribe({
      next: (expenses) => {
        this.expenses = expenses;
      },
      error: (error) => {
        console.error('Error loading expenses:', error);
      }
    });
  }
  groupId: string = '';
  balances: Balance[] = [];
  currentUser: User | null = null;
  
  newExpense: CreateExpense = {
    amount: 0,
    description: ''
  };
  
  newPayment: CreatePayment = {
    to_user_id: '',
    amount: 0
  };
  
  isLoadingBalances = false;
  isAddingExpense = false;
  isRecordingPayment = false;
  
  expenseError = '';
  paymentError = '';
  expenseSuccess = '';
  paymentSuccess = '';

  constructor(
    private route: ActivatedRoute,
    private router: Router,
    private apiService: ApiService
  ) {}

  ngOnInit() {
    this.currentUser = this.apiService.getCurrentUser();
    this.groupId = this.route.snapshot.paramMap.get('id') || '';
    if (this.groupId) {
      this.loadBalances();
      this.loadExpenses();
    } else {
      this.router.navigate(['/dashboard']);
    }
  }

  // Helper methods for robust floating-point comparisons
  // These methods use epsilon-based comparison to handle precision errors
  private isPositive(amount: number): boolean {
    return amount > this.EPSILON;
  }

  private isNegative(amount: number): boolean {
    return amount < -this.EPSILON;
  }

  private isZero(amount: number): boolean {
    return Math.abs(amount) <= this.EPSILON;
  }

  private roundToTwoDecimals(amount: number): number {
    return Math.round(amount * 100) / 100;
  }

  loadBalances() {
    this.isLoadingBalances = true;
    this.apiService.getGroupBalances(this.groupId).subscribe({
      next: (balances) => {
        this.balances = balances;
        this.isLoadingBalances = false;
      },
      error: (error) => {
        console.error('Error loading balances:', error);
        this.isLoadingBalances = false;
        alert('Failed to load group balances. Please try again.');
      }
    });
  }

  addExpense() {
    if (!this.newExpense.description || this.newExpense.amount <= 0) {
      this.expenseError = 'Please enter a valid description and amount';
      return;
    }

    this.isAddingExpense = true;
    this.expenseError = '';
    this.expenseSuccess = '';

    this.apiService.addExpense(this.groupId, this.newExpense).subscribe({
      next: () => {
        this.isAddingExpense = false;
        this.expenseSuccess = `Expense "${this.newExpense.description}" added successfully!`;
        this.newExpense = { amount: 0, description: '' };
        this.loadBalances(); // Refresh balances
      },
      error: (error) => {
        this.isAddingExpense = false;
        this.expenseError = error.error?.message || 'Failed to add expense. Please try again.';
        console.error('Add expense error:', error);
      }
    });
  }

  recordPayment() {
    // Only allow current user to pay if they have negative balance
    const currentUserBalance = this.balances.find(b => b.user_id === this.currentUser?.id);
    if (!currentUserBalance || !this.isNegative(currentUserBalance.balance)) {
      this.paymentError = 'Only members who owe money (negative balance) can record payments.';
      return;
    }
    // Only allow payment to users with positive balance
    const toUserBalance = this.balances.find(b => b.user_id === this.newPayment.to_user_id);
    if (!toUserBalance || !this.isPositive(toUserBalance.balance)) {
      this.paymentError = 'You can only pay members who are owed money (positive balance).';
      return;
    }
    if (!this.newPayment.to_user_id || this.newPayment.amount <= 0) {
      this.paymentError = 'Please select a user and enter a valid amount';
      return;
    }
    // Limit payment to min(abs(negative balance), positive balance)
    const maxPayable = Math.min(Math.abs(currentUserBalance.balance), toUserBalance.balance);
    
    // Round to avoid floating point precision issues
    const roundedMaxPayable = this.roundToTwoDecimals(maxPayable);
    const roundedPaymentAmount = this.roundToTwoDecimals(this.newPayment.amount);
    
    if (roundedPaymentAmount > roundedMaxPayable + this.EPSILON) {
      this.paymentError = `You cannot pay more than ₹${roundedMaxPayable.toFixed(2)} (your debt amount).`;
      return;
    }

    this.isRecordingPayment = true;
    this.paymentError = '';
    this.paymentSuccess = '';

    this.apiService.recordPayment(this.groupId, this.newPayment).subscribe({
      next: () => {
        this.isRecordingPayment = false;
        const toUser = this.getUserById(this.newPayment.to_user_id);
        this.paymentSuccess = `Payment of ₹${this.newPayment.amount} to ${toUser?.username} recorded successfully!`;
        this.newPayment = { to_user_id: '', amount: 0 };
        this.loadBalances(); // Refresh balances
      },
      error: (error) => {
        this.isRecordingPayment = false;
        this.paymentError = error.error?.message || 'Failed to record payment. Please try again.';
        console.error('Record payment error:', error);
      }
    });
  }

  getSelectedRecipientMax(): number {
    const toUserBalance = this.balances.find(b => b.user_id === this.newPayment.to_user_id);
    const currentUserBalance = this.balances.find(b => b.user_id === this.currentUser?.id);
    if (toUserBalance && this.isPositive(toUserBalance.balance) && 
        currentUserBalance && this.isNegative(currentUserBalance.balance)) {
      const maxAmount = Math.min(toUserBalance.balance, Math.abs(currentUserBalance.balance));
      return this.roundToTwoDecimals(maxAmount);
    }
    return 0;
  }

  getUserById(userId: string): Balance | undefined {
    return this.balances.find(balance => balance.user_id === userId);
  }

  getPositiveBalances(): Balance[] {
    // Only members with positive balance (more than epsilon)
    return this.balances.filter(balance => this.isPositive(balance.balance));
  }

  getCurrentUserNegativeBalance(): boolean {
    const currentUserBalance = this.balances.find(b => b.user_id === this.currentUser?.id);
    // Only show payment section if user owes more than epsilon
    return !!currentUserBalance && this.isNegative(currentUserBalance.balance);
  }

  getNegativeBalances(): Balance[] {
    return this.balances.filter(balance => this.isNegative(balance.balance));
  }

  getZeroBalances(): Balance[] {
    return this.balances.filter(balance => this.isZero(balance.balance));
  }

  getSettlementSuggestions(): { from: Balance; to: Balance; amount: number }[] {
    const debtors = this.getNegativeBalances().map(b => ({ ...b }));
    const creditors = this.getPositiveBalances().map(b => ({ ...b }));
    const suggestions: { from: Balance; to: Balance; amount: number }[] = [];

    // Simple settlement algorithm with epsilon-based comparisons
    for (let debtor of debtors) {
      for (let creditor of creditors) {
        if (this.isNegative(debtor.balance) && this.isPositive(creditor.balance)) {
          const amount = Math.min(Math.abs(debtor.balance), creditor.balance);
          const roundedAmount = this.roundToTwoDecimals(amount);
          
          if (roundedAmount > this.EPSILON) {
            suggestions.push({
              from: debtor,
              to: creditor,
              amount: roundedAmount
            });
            debtor.balance += roundedAmount;
            creditor.balance -= roundedAmount;
          }
        }
      }
    }

    return suggestions;
  }

  formatCurrency(amount: number): string {
    return new Intl.NumberFormat('en-IN', {
      style: 'currency',
      currency: 'INR'
    }).format(amount);
  }

  formatAbsoluteCurrency(amount: number): string {
    return new Intl.NumberFormat('en-IN', {
      style: 'currency',
      currency: 'INR'
    }).format(Math.abs(amount));
  }

  goBack() {
    this.router.navigate(['/dashboard']);
  }
}
