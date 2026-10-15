import type {StorageFullReport} from '$lib/api/types';

export const storageActivity = $state({
    report: null as StorageFullReport | null,
    loading: false,
    cleanup: null as 'logs' | 'cache' | null,
    error: ''
});
