import { request as __request } from '../request';
import { CancelablePromise } from '../CancelablePromise';

export interface B {
  b: C;
}

export interface C {
  c: number;
}

export interface A {
  a: B;
}

export function request(json: [B, A], path: C): CancelablePromise<C> {
  return __request(
    { url: 'http://localhost:3000' },
    {
      method: 'POST',
      url: '/backend/a',
      body: JSON.stringify(json),
      path,
    }
  );
}
