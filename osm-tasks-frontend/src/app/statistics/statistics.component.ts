import { Component, OnInit } from '@angular/core';
import { CollectedBenchmarks } from '../../../generated/models/collected-benchmarks';
import * as am4core from '@amcharts/amcharts4/core';
import * as am4charts from '@amcharts/amcharts4/charts';
import { ApiService } from '../../../generated/services/api.service';
import am4themes_material from "@amcharts/amcharts4/themes/material";
import { ActivatedRoute } from '@angular/router';
import { switchMap } from 'rxjs/operators';

@Component({
  selector: 'app-statistics',
  templateUrl: './statistics.component.html',
  styleUrls: ['./statistics.component.css']
})
export class StatisticsComponent implements OnInit {
  mode = 'time'

  constructor(private apiService: ApiService, private route: ActivatedRoute) {
  }

  ngOnInit(): void {
    am4core.useTheme(am4themes_material);

    let param = this.route.queryParams['value']['mode'];
    if (param != null) {
      this.mode = param;
    }

    this.requestBenchmarkResult()
  }

  requestBenchmarkResult() {
    this.apiService.benchmarkResults().subscribe(
      (data: CollectedBenchmarks) => {
        this.generateChart(data);
      },
      (error) => console.log(error));
  }

  generateChart(data: CollectedBenchmarks) {
    let chart = am4core.create("chart", am4charts.XYChart);
    chart.numberFormatter.numberFormat = "#a";
    chart.numberFormatter.bigNumberPrefixes = [
      { "number": 1e+3, "suffix": "x10³" },
      { "number": 1e+6, "suffix": "x10⁶" },
      { "number": 1e+9, "suffix": "x10⁹" }
    ];
    chart.data = data as any

    let yAxis = chart.yAxes.push(new am4charts.ValueAxis());
    if (this.mode == 'time') {
      yAxis.title.text = "Nanoseconds"
    } else if (this.mode == 'amount_nodes_popped') {
      yAxis.title.text = "Number nodes popped"
    } else {
      yAxis.title.text = this.mode
    }
    let xAxis = chart.xAxes.push(new am4charts.ValueAxis());
    xAxis.title.text = "Query ID"

    this.createSeries(chart, "Dijkstra", data.dijkstra.results);
    this.createSeries(chart, "Bidirectional Dijkstra", data.bd_dijkstra.results);
    this.createSeries(chart, "A Star", data.a_star.results);
    this.createSeries(chart, "CH", data.ch.results);

    chart.cursor = new am4charts.XYCursor();
    chart.legend = new am4charts.Legend();
  }

  createSeries(chart: am4charts.XYChart, name: string, data: any) {
    let series = chart.series.push(new am4charts.LineSeries());
    series.name = name
    series.data = data
    series.dataFields.valueY = this.mode
    series.dataFields.valueX = "query_id"
    series.tooltipText = "{name} {query_id}: \n {time}";
    series.strokeWidth = 3
  }
}
