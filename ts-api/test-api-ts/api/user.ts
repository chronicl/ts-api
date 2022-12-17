import { request as __request } from '../request';
import { CancelablePromise } from '../CancelablePromise';

export function request(json: string): CancelablePromise<number> {
    return __request(
        { url: 'http://localhost:3000' },
        {
            method: 'GET',
            url: '/user',
            body: JSON.stringify(json)
        }
    );
}