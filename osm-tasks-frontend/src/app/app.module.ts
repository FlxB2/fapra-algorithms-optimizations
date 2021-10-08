import { BrowserModule } from '@angular/platform-browser';
import { NgModule } from '@angular/core';

import { AppComponent } from './app.component';
import { LeafletModule } from '@asymmetrik/ngx-leaflet';
import { HttpClientModule } from '@angular/common/http';
import { ApiModule } from '../../generated/api.module';
import { FormsModule } from '@angular/forms';
import { AlertModule } from 'ngx-bootstrap/alert';
import { ModalModule } from 'ngx-bootstrap/modal';
import { StatisticsComponent } from './statistics/statistics.component';
import { appRoutingModule } from './app.routing';
import { MapComponent } from './map/map.component';
import { BsDropdownModule } from 'ngx-bootstrap/dropdown';
import { BrowserAnimationsModule } from '@angular/platform-browser/animations';

// @ts-ignore
@NgModule({
  declarations: [
    AppComponent,
    StatisticsComponent,
    MapComponent
  ],
  imports: [
    appRoutingModule,
    BrowserAnimationsModule,
    BrowserModule,
    LeafletModule,
    ApiModule.forRoot({ rootUrl: 'http://localhost:8000' }),
    HttpClientModule,
    FormsModule,
    AlertModule.forRoot(),
    ModalModule.forRoot(),
    BsDropdownModule
  ],
  bootstrap: [AppComponent]
})
export class AppModule { }
