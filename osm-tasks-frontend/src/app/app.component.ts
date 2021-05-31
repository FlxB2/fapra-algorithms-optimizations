import { AfterViewInit, Component } from '@angular/core';
import { map, marker, latLng, MapOptions, Polygon, tileLayer, ZoomAnimEvent } from 'leaflet';
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

  startLat: number = 0.0;
  startLon: number = 0.0;
  endLat: number = 0.0;
  endLon: number = 0.0;
  jobId: number = 0;

  private map;

  initMap(): void {
    this.map = map('map', this.options)
  }

  ngAfterViewInit(): void {
    this.initMap()
    console.log("init")
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
    this.apiService.buildGraph().subscribe(data => console.log(data));
  }

  requestRoute() {
    this.apiService.route({
      lat_start: this.startLat, lon_start: this.startLon, lat_end: this.endLat, lon_end: this.endLon
    }).subscribe(data => console.log(data));
  }

  requestResult() {
    this.apiService.jobResult({ id: this.jobId }).subscribe((res: ShipRoute) =>
      res.nodes.forEach((node) => {
        marker([node.lat, node.lon]).addTo(this.map);
      })
    );
  }
}
