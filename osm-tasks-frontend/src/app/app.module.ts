import { BrowserModule } from '@angular/platform-browser';
import { NgModule } from '@angular/core';

import { AppComponent } from './app.component';
import { LeafletModule } from '@asymmetrik/ngx-leaflet';
import { HttpClientModule } from '@angular/common/http';
import { ApiModule } from '../../generated/api.module';
import { FormsModule } from '@angular/forms';
import { AlertModule } from 'ngx-bootstrap/alert';
import { ModalModule } from 'ngx-bootstrap/modal';

// @ts-ignore
@NgModule({
  declarations: [
    AppComponent
  ],
  imports: [
    BrowserModule,
    LeafletModule,
    ApiModule.forRoot({ rootUrl: 'http://localhost:8000' }),
    HttpClientModule,
    FormsModule,
    AlertModule.forRoot(),
    ModalModule.forRoot()
  ],
  bootstrap: [AppComponent]
})
export class AppModule { }
