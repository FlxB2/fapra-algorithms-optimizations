/* tslint:disable */
/* eslint-disable */
import { Injectable } from '@angular/core';
import { HttpClient, HttpResponse } from '@angular/common/http';
import { BaseService } from '../base-service';
import { ApiConfiguration } from '../api-configuration';
import { StrictHttpResponse } from '../strict-http-response';
import { RequestBuilder } from '../request-builder';
import { Observable } from 'rxjs';
import { map, filter } from 'rxjs/operators';

import { RouteRequest } from '../models/route-request';
import { ShipRoute } from '../models/ship-route';

@Injectable({
  providedIn: 'root',
})
export class ApiService extends BaseService {
  constructor(
    config: ApiConfiguration,
    http: HttpClient
  ) {
    super(config, http);
  }

  /**
   * Path part for operation buildGraph
   */
  static readonly BuildGraphPath = '/build_graph';

  /**
   * This method provides access to the full `HttpResponse`, allowing access to response headers.
   * To access only the response body, use `buildGraph()` instead.
   *
   * This method doesn't expect any request body.
   */
  buildGraph$Response(params?: {
  }): Observable<StrictHttpResponse<void>> {

    const rb = new RequestBuilder(this.rootUrl, ApiService.BuildGraphPath, 'post');
    if (params) {
    }

    return this.http.request(rb.build({
      responseType: 'text',
      accept: '*/*'
    })).pipe(
      filter((r: any) => r instanceof HttpResponse),
      map((r: HttpResponse<any>) => {
        return (r as HttpResponse<any>).clone({ body: undefined }) as StrictHttpResponse<void>;
      })
    );
  }

  /**
   * This method provides access to only to the response body.
   * To access the full response (for headers, for example), `buildGraph$Response()` instead.
   *
   * This method doesn't expect any request body.
   */
  buildGraph(params?: {
  }): Observable<void> {

    return this.buildGraph$Response(params).pipe(
      map((r: StrictHttpResponse<void>) => r.body as void)
    );
  }

  /**
   * Path part for operation jobResult
   */
  static readonly JobResultPath = '/jobResult/{id}';

  /**
   * This method provides access to the full `HttpResponse`, allowing access to response headers.
   * To access only the response body, use `jobResult()` instead.
   *
   * This method doesn't expect any request body.
   */
  jobResult$Response(params: {
    id: number;
  }): Observable<StrictHttpResponse<ShipRoute>> {

    const rb = new RequestBuilder(this.rootUrl, ApiService.JobResultPath, 'get');
    if (params) {
      rb.path('id', params.id, {});
    }

    return this.http.request(rb.build({
      responseType: 'json',
      accept: 'application/json'
    })).pipe(
      filter((r: any) => r instanceof HttpResponse),
      map((r: HttpResponse<any>) => {
        return r as StrictHttpResponse<ShipRoute>;
      })
    );
  }

  /**
   * This method provides access to only to the response body.
   * To access the full response (for headers, for example), `jobResult$Response()` instead.
   *
   * This method doesn't expect any request body.
   */
  jobResult(params: {
    id: number;
  }): Observable<ShipRoute> {

    return this.jobResult$Response(params).pipe(
      map((r: StrictHttpResponse<ShipRoute>) => r.body as ShipRoute)
    );
  }

  /**
   * Path part for operation jobStatus
   */
  static readonly JobStatusPath = '/jobStatus/{id}';

  /**
   * This method provides access to the full `HttpResponse`, allowing access to response headers.
   * To access only the response body, use `jobStatus()` instead.
   *
   * This method doesn't expect any request body.
   */
  jobStatus$Response(params: {
    id: number;
  }): Observable<StrictHttpResponse<boolean>> {

    const rb = new RequestBuilder(this.rootUrl, ApiService.JobStatusPath, 'get');
    if (params) {
      rb.path('id', params.id, {});
    }

    return this.http.request(rb.build({
      responseType: 'json',
      accept: 'application/json'
    })).pipe(
      filter((r: any) => r instanceof HttpResponse),
      map((r: HttpResponse<any>) => {
        return (r as HttpResponse<any>).clone({ body: String((r as HttpResponse<any>).body) === 'true' }) as StrictHttpResponse<boolean>;
      })
    );
  }

  /**
   * This method provides access to only to the response body.
   * To access the full response (for headers, for example), `jobStatus$Response()` instead.
   *
   * This method doesn't expect any request body.
   */
  jobStatus(params: {
    id: number;
  }): Observable<boolean> {

    return this.jobStatus$Response(params).pipe(
      map((r: StrictHttpResponse<boolean>) => r.body as boolean)
    );
  }

  /**
   * Path part for operation route
   */
  static readonly RoutePath = '/route';

  /**
   * This method provides access to the full `HttpResponse`, allowing access to response headers.
   * To access only the response body, use `route()` instead.
   *
   * This method sends `application/json` and handles request body of type `application/json`.
   */
  route$Response(params: {
    body: RouteRequest
  }): Observable<StrictHttpResponse<number>> {

    const rb = new RequestBuilder(this.rootUrl, ApiService.RoutePath, 'post');
    if (params) {
      rb.body(params.body, 'application/json');
    }

    return this.http.request(rb.build({
      responseType: 'json',
      accept: 'application/json'
    })).pipe(
      filter((r: any) => r instanceof HttpResponse),
      map((r: HttpResponse<any>) => {
        return (r as HttpResponse<any>).clone({ body: parseFloat(String((r as HttpResponse<any>).body)) }) as StrictHttpResponse<number>;
      })
    );
  }

  /**
   * This method provides access to only to the response body.
   * To access the full response (for headers, for example), `route$Response()` instead.
   *
   * This method sends `application/json` and handles request body of type `application/json`.
   */
  route(params: {
    body: RouteRequest
  }): Observable<number> {

    return this.route$Response(params).pipe(
      map((r: StrictHttpResponse<number>) => r.body as number)
    );
  }

  /**
   * Path part for operation test
   */
  static readonly TestPath = '/test_graph';

  /**
   * This method provides access to the full `HttpResponse`, allowing access to response headers.
   * To access only the response body, use `test()` instead.
   *
   * This method doesn't expect any request body.
   */
  test$Response(params?: {
  }): Observable<StrictHttpResponse<number>> {

    const rb = new RequestBuilder(this.rootUrl, ApiService.TestPath, 'get');
    if (params) {
    }

    return this.http.request(rb.build({
      responseType: 'json',
      accept: 'application/json'
    })).pipe(
      filter((r: any) => r instanceof HttpResponse),
      map((r: HttpResponse<any>) => {
        return (r as HttpResponse<any>).clone({ body: parseFloat(String((r as HttpResponse<any>).body)) }) as StrictHttpResponse<number>;
      })
    );
  }

  /**
   * This method provides access to only to the response body.
   * To access the full response (for headers, for example), `test$Response()` instead.
   *
   * This method doesn't expect any request body.
   */
  test(params?: {
  }): Observable<number> {

    return this.test$Response(params).pipe(
      map((r: StrictHttpResponse<number>) => r.body as number)
    );
  }

}
