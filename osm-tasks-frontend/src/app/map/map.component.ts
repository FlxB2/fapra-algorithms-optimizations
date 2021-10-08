import { AfterViewInit, Component, OnInit, TemplateRef, ViewChild } from '@angular/core';
import {
  DomEvent,
  DomUtil,
  Geodesic,
  icon,
  latLng,
  LatLngExpression,
  LatLngTuple,
  map,
  MapOptions,
  marker,
  Marker,
  Polyline,
  popup,
  tileLayer
} from 'leaflet';
import { ApiService } from '../../../generated/services/api.service';
import { ShipRoute } from '../../../generated/models/ship-route';
import 'leaflet.geodesic';
import { BsModalRef, BsModalService } from 'ngx-bootstrap/modal';
import { CollectedBenchmarks } from '../../../generated/models/collected-benchmarks';
import * as am4core from "@amcharts/amcharts4/core";
import * as am4charts from "@amcharts/amcharts4/charts";
import { Router } from '@angular/router';

// From https://www.iconfinder.com/icons/4908137/destination_ensign_flag_pole_signal_icon
// No link back required
const destinationIcon = 'data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiA/PjxzdmcgZGF0YS1uYW1lPSJMYXllciAxIiBp' +
  'ZD0iTGF5ZXJfMSIgdmlld0JveD0iMCAwIDI0IDI0IiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjx0aXRsZS8+PHBhdG' +
  'ggZD0iTTYsMTMuM2wxLjIxLjU0QTcuMjIsNy4yMiwwLDAsMCwxMi44LDE0YTUuMTcsNS4xNywwLDAsMSw0LjIzLjE4bDMsMS40OFY1LjM4' +
  'bC0yLjA4LTFhNy4xOCw3LjE4LDAsMCwwLTUuODctLjI0QTUuMiw1LjIsMCwwLDEsOCw0TDYsMy4xMVYySDRWMjJINloiLz48L3N2Zz4=';

@Component({
  selector: 'app-map',
  templateUrl: './map.component.html',
  styleUrls: ['./map.component.css']
})
export class MapComponent implements OnInit {
  modalRefHelp: BsModalRef;
  modalRefData: BsModalRef;

  @ViewChild('data') public templateref: TemplateRef<any>;

  constructor(private apiService: ApiService, private modalService: BsModalService) {
  }

  title = 'osm-tasks-frontend';

  alerts: any[] = [];
  startLat = 0.0;
  startLon = 0.0;
  endLat = 0.0;
  endLon = 0.0;
  jobId = 0;
  benchmarkRuns = 10;

  currentRoute: Polyline;
  markerStart: Marker;
  markerStop: Marker;

  private map;

  options: MapOptions = {
    layers: [tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      opacity: 1.0,
      maxZoom: 19,
      detectRetina: true,
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
      noWrap: true
    })],
    zoom: 3,
    center: latLng(51.1657, 10.4515)
  };

  openModal(template: TemplateRef<any>) {
    this.modalRefHelp = this.modalService.show(template);
  }

  initMap(): void {
    this.map = map('map', this.options);
    this.map.on('click', (e: any) => {
      this.defineYourWaypointOnClick(e);
    });
  }

  ngAfterViewInit(): void {
    this.initMap();
  }

  buildGraph() {
    this.apiService.buildGraph({ num_nodes: 1000000 }).subscribe(
      _ => this.showAlert('Building graph.. this might take a while', 'info'));
  }

  defineYourWaypointOnClick(e: any) {
    // Code taken from: https://stackoverflow.com/questions/42599445/adding-buttons-inside-leaflet-popup
    const choicePopUp = popup();
    const container = DomUtil.create('div');
    const startBtn = this.createButton('Start from this location', container);
    const destBtn = this.createButton('Go to this location', container);

    choicePopUp
      .setLatLng(e.latlng)
      .setContent(container)
      .openOn(this.map);

    DomEvent.on(startBtn, 'click', () => {
      this.map.closePopup();
      this.startLat = e.latlng.lat;
      this.startLon = e.latlng.lng;
      this.updateMarkers();
    });

    DomEvent.on(destBtn, 'click', () => {
      this.map.closePopup();
      this.endLat = e.latlng.lat;
      this.endLon = e.latlng.lng;
      this.updateMarkers();
    });
  }

  updateMarkers(startLat = this.startLat, startLon = this.startLon, endLat = this.endLat, endLon = this.endLon) {
    if (this.markerStart != null) {
      this.map.removeLayer(this.markerStart);
    }
    this.markerStart = this.getMarker([startLat, startLon]);
    this.markerStart.addTo(this.map);
    if (this.markerStop != null) {
      this.map.removeLayer(this.markerStop);
    }
    this.markerStop = this.getDestinationMarker([endLat, endLon]);
    this.markerStop.addTo(this.map);
  }

  createButton(label: string, container: any) {
    const btn = DomUtil.create('button', '', container);
    btn.setAttribute('type', 'button');
    btn.innerHTML = label;
    return btn;
  }

  requestBenchmark() {
    this.apiService.startBenchmark({ nmb_queries: this.benchmarkRuns }).subscribe(
      () => this.showAlert('Success, benchmark running', 'info'),
      (error) => this.showAlert(error.toString(), 'danger')
    )
  }

  requestRoute() {
    this.apiService.route({
      lat_start: this.startLat, lon_start: this.startLon, lat_end: this.endLat, lon_end: this.endLon
    }).subscribe(
      data => {
        this.showAlert('calculating result, jobId: ' + data, 'info');
        this.jobId = data;
        // try to fetch the result after one second
        setTimeout(() => {
          this.requestResult();
        }, 1000);
      },
      () => this.showAlert('Could not calculate result did you build the graph?', 'danger'));
  }

  requestResult() {
    this.apiService.jobResult({ id: this.jobId }).subscribe((res: ShipRoute) => {
        if (this.currentRoute != null) {
          this.map.removeLayer(this.currentRoute);
        }
        const lines: LatLngTuple[][] = [[]];
        let lastLon = 0;
        res.nodes.forEach((node) => {
          if ((Math.floor(node.lon) === 180 && lastLon < 0) || (Math.floor(node.lon) === -180 && lastLon > 0)) {
            // Crossed 180 degree line -> split line
            lines.push([]);
          }
          lines[lines.length - 1].push([node.lat, node.lon]);
          lastLon = node.lon;
        });
        // Use Geodesic that the lines will wrapped around 180 degree
        this.currentRoute = new Geodesic(lines).addTo(this.map);
        this.updateMarkers(res.nodes[0].lat, res.nodes[0].lon, res.nodes[res.nodes.length - 1].lat, res.nodes[res.nodes.length - 1].lon);
        this.map.addLayer(this.currentRoute);
        this.showAlert('Success! Route length in m: ' + res.distance, 'info');
      }, () => this.showAlert('Could not fetch result did you build the graph?' +
        ' And did you check the id? Calculating a route might take a while', 'danger')
    );
  }

  showAlert(msg: string, type: string) {
    this.alerts.push({
      type,
      msg,
      timeout: 5000
    });
  }

  getDestinationMarker(location: LatLngExpression): Marker {
    return marker(location, {
      icon: icon({
        iconSize: [41, 41],
        iconAnchor: [7, 38],
        iconUrl: destinationIcon,
        shadowUrl: 'assets/marker-shadow.png'
      })
    });
  }

  getMarker(location: LatLngExpression): Marker {
    return marker(location, {
      icon: icon({
        iconSize: [25, 41],
        iconAnchor: [13.5, 41],
        iconUrl: 'assets/marker-icon.png',
        shadowUrl: 'assets/marker-shadow.png'
      })
    });
  }

  ngOnInit(): void {
  }
}
