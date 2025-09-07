export interface Group {
  id: string;
  name: string;
  created_by: string;
  created_at: string;
}

export interface CreateGroupRequest {
  name: string;
  user_ids: string[];
}

export interface Balance {
  user_id: string;
  username: string;
  balance: number;
}

export interface CreateExpense {
  amount: number;
  description: string;
}

export interface CreatePayment {
  to_user_id: string;
  amount: number;
}
