import { AfterViewInit, Component } from '@angular/core';
import {
  map, marker, latLng, MapOptions, tileLayer, polyline, LatLngExpression, Polyline, icon, Marker
} from 'leaflet';
import { ApiService } from '../../generated/services/api.service';
import { ShipRoute } from '../../generated/models/ship-route';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.css']
})
export class AppComponent implements AfterViewInit {

  constructor(private apiService: ApiService) {
  }

  title = 'osm-tasks-frontend';

  alerts: any[] = [];
  startLat: number = 0.0;
  startLon: number = 0.0;
  endLat: number = 0.0;
  endLon: number = 0.0;
  jobId: number = 0;

  currentRoute: Polyline;
  markerStart: Marker;
  markerStop: Marker;

  private map;

  initMap(): void {
    this.map = map('map', this.options)
  }

  ngAfterViewInit(): void {
    this.initMap()
  }

  options: MapOptions = {
    layers: [tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      opacity: 1.0,
      maxZoom: 19,
      detectRetina: true,
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
    })],
    zoom: 3,
    center: latLng(51.1657, 10.4515)
  };

  buildGraph() {
    this.apiService.buildGraph().subscribe(
      _ => this.showAlert("Building graph.. this might take a while", "info"));
  }

  requestRoute() {
    this.apiService.route({
      lat_start: this.startLat, lon_start: this.startLon, lat_end: this.endLat, lon_end: this.endLon
    }).subscribe(
      data => this.showAlert("calculating result, jobId: " + data, "info"),
      () => this.showAlert("Could not calculate result did you build the graph?", "danger"));
  }

  requestResult() {
    this.apiService.jobResult({ id: this.jobId }).subscribe((res: ShipRoute) => {
        if (this.currentRoute != null) {
          this.map.removeLayer(this.currentRoute);
          this.map.removeLayer(this.markerStart);
          this.map.removeLayer(this.markerStop);
        }
        const array: LatLngExpression[] = [];
        this.markerStart = this.getMarker([res.nodes[0].lat, res.nodes[0].lon])
        this.markerStop = this.getMarker([res.nodes[res.nodes.length - 1].lat, res.nodes[res.nodes.length - 1].lon])
        this.markerStart.addTo(this.map)
        this.markerStop.addTo(this.map)
        res.nodes.forEach((node) => {
          array.push([node.lat, node.lon])
        });
        this.currentRoute = polyline(array)
        this.map.addLayer(this.currentRoute);
        this.showAlert("Success! Route length in m: " + res.distance, "info");
      }, () => this.showAlert("Could not fetch result did you build the graph? And did you check the id? Calculating a route might take a while", "danger")
    );
  }

  showAlert(msg: String, type: String) {
    this.alerts.push({
      type: type,
      msg: msg,
      timeout: 5000
    });
  }

  getMarker(location: LatLngExpression): Marker {
    return marker(location, {
      icon: icon({
        iconSize: [25, 41],
        iconAnchor: [13, 41],
        iconUrl: 'assets/marker-icon.png',
        shadowUrl: 'assets/marker-shadow.png'
      })
    })
  }
}
