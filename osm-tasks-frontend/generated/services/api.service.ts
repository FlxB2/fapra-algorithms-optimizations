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

import { CollectedBenchmarks } from '../models/collected-benchmarks';
import { Response } from '../models/response';
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
   * Path part for operation benchmarkResults
   */
  static readonly BenchmarkResultsPath = '/benchmarkResults';

  /**
   * This method provides access to the full `HttpResponse`, allowing access to response headers.
   * To access only the response body, use `benchmarkResults()` instead.
   *
   * This method doesn't expect any request body.
   */
  benchmarkResults$Response(params?: {
  }): Observable<StrictHttpResponse<CollectedBenchmarks>> {

    const rb = new RequestBuilder(this.rootUrl, ApiService.BenchmarkResultsPath, 'get');
    if (params) {
    }

    return this.http.request(rb.build({
      responseType: 'json',
      accept: 'application/json'
    })).pipe(
      filter((r: any) => r instanceof HttpResponse),
      map((r: HttpResponse<any>) => {
        return r as StrictHttpResponse<CollectedBenchmarks>;
      })
    );
  }

  /**
   * This method provides access to only to the response body.
   * To access the full response (for headers, for example), `benchmarkResults$Response()` instead.
   *
   * This method doesn't expect any request body.
   */
  benchmarkResults(params?: {
  }): Observable<CollectedBenchmarks> {

    return this.benchmarkResults$Response(params).pipe(
      map((r: StrictHttpResponse<CollectedBenchmarks>) => r.body as CollectedBenchmarks)
    );
  }

  /**
   * Path part for operation buildGraph
   */
  static readonly BuildGraphPath = '/buildGraph';

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
   * Path part for operation checkBenchmark
   */
  static readonly CheckBenchmarkPath = '/isBenchmarkRunning';

  /**
   * This method provides access to the full `HttpResponse`, allowing access to response headers.
   * To access only the response body, use `checkBenchmark()` instead.
   *
   * This method doesn't expect any request body.
   */
  checkBenchmark$Response(params?: {
  }): Observable<StrictHttpResponse<boolean>> {

    const rb = new RequestBuilder(this.rootUrl, ApiService.CheckBenchmarkPath, 'get');
    if (params) {
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
   * To access the full response (for headers, for example), `checkBenchmark$Response()` instead.
   *
   * This method doesn't expect any request body.
   */
  checkBenchmark(params?: {
  }): Observable<boolean> {

    return this.checkBenchmark$Response(params).pipe(
      map((r: StrictHttpResponse<boolean>) => r.body as boolean)
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
  static readonly JobStatusPath = '/jobStatus';

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
      rb.query('id', params.id, {});
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
   * This method doesn't expect any request body.
   */
  route$Response(params: {
    lat_start: number;
    lon_start: number;
    lat_end: number;
    lon_end: number;
  }): Observable<StrictHttpResponse<number>> {

    const rb = new RequestBuilder(this.rootUrl, ApiService.RoutePath, 'get');
    if (params) {
      rb.query('lat_start', params.lat_start, {});
      rb.query('lon_start', params.lon_start, {});
      rb.query('lat_end', params.lat_end, {});
      rb.query('lon_end', params.lon_end, {});
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
   * This method doesn't expect any request body.
   */
  route(params: {
    lat_start: number;
    lon_start: number;
    lat_end: number;
    lon_end: number;
  }): Observable<number> {

    return this.route$Response(params).pipe(
      map((r: StrictHttpResponse<number>) => r.body as number)
    );
  }

  /**
   * Path part for operation startBenchmark
   */
  static readonly StartBenchmarkPath = '/startBenchmark';

  /**
   * This method provides access to the full `HttpResponse`, allowing access to response headers.
   * To access only the response body, use `startBenchmark()` instead.
   *
   * This method doesn't expect any request body.
   */
  startBenchmark$Response(params: {
    nmb_queries: number;
  }): Observable<StrictHttpResponse<Response>> {

    const rb = new RequestBuilder(this.rootUrl, ApiService.StartBenchmarkPath, 'post');
    if (params) {
      rb.query('nmb_queries', params.nmb_queries, {});
    }

    return this.http.request(rb.build({
      responseType: 'json',
      accept: 'application/json'
    })).pipe(
      filter((r: any) => r instanceof HttpResponse),
      map((r: HttpResponse<any>) => {
        return r as StrictHttpResponse<Response>;
      })
    );
  }

  /**
   * This method provides access to only to the response body.
   * To access the full response (for headers, for example), `startBenchmark$Response()` instead.
   *
   * This method doesn't expect any request body.
   */
  startBenchmark(params: {
    nmb_queries: number;
  }): Observable<Response> {

    return this.startBenchmark$Response(params).pipe(
      map((r: StrictHttpResponse<Response>) => r.body as Response)
    );
  }

  /**
   * Path part for operation test
   */
  static readonly TestPath = '/testGraph';

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
