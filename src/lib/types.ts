export interface AccountSummary {
  accountKey: string;
  email: string;
  alias: string;
  accountName: string | null;
  displayName: string;
  plan: string | null;
  isActive: boolean;
  usage5hRemaining: number | null;
  usageWeeklyRemaining: number | null;
  hasAuthSnapshot: boolean;
  lastUsedAt: number | null;
}

export interface PanelState {
  codexHome: string;
  registryPath: string;
  hasRegistry: boolean;
  activeAccountKey: string | null;
  autoSwitchEnabled: boolean;
  apiUsageEnabled: boolean;
  accounts: AccountSummary[];
}
