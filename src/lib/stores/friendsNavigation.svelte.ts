type FriendsTab = 'all' | 'online' | 'pending' | 'add' | 'p2p';

export const friendsNavigation = $state<{ tab: FriendsTab }>({ tab: 'all' });
