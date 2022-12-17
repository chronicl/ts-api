import { request as __request } from '../request';
import { CancelablePromise } from '../CancelablePromise';

export interface RequestType {
  a: number;
}

export interface ResponseType {
  a: number;
}

export interface Path {
  post: number;
}

export interface Query {}

export function request(json: RequestType, path: Path, query: Query): CancelablePromise<ResponseType> {
  return __request(
    { url: 'http://localhost:3000' },
    {
      method: 'POST',
      url: 'backend/user/{post}',
      body: JSON.stringify(json),
      path,
      query,
    }
  );
}
