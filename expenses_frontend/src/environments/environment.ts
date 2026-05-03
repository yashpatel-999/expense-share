declare const window: any;

export const environment = {
  production: true,
  apiUrl:
    (typeof window !== 'undefined' && window.apiUrl) ||
    'http://localhost:8080/api',
};
