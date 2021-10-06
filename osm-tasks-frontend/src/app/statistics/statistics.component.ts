import { Component, OnInit } from '@angular/core';
import { CollectedBenchmarks } from '../../../generated/models/collected-benchmarks';
import * as am4core from '@amcharts/amcharts4/core';
import * as am4charts from '@amcharts/amcharts4/charts';
import { ApiService } from '../../../generated/services/api.service';
import am4themes_material from "@amcharts/amcharts4/themes/material";

@Component({
  selector: 'app-statistics',
  templateUrl: './statistics.component.html',
  styleUrls: ['./statistics.component.css']
})
export class StatisticsComponent implements OnInit {

  constructor(private apiService: ApiService) {
  }

  ngOnInit(): void {
    am4core.useTheme(am4themes_material);
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
    let dijkstra_results = data.dijkstra.results
    let bd_dijkstra_results = data.bd_dijkstra.results
    let a_star_results = data.a_star.results

    let chart = am4core.create("chart", am4charts.XYChart);
    chart.numberFormatter.numberFormat = "#a";
    chart.numberFormatter.bigNumberPrefixes = [
      { "number": 1e+3, "suffix": "x10³" },
      { "number": 1e+6, "suffix": "x10⁶" },
      { "number": 1e+9, "suffix": "x10⁹" }
    ];
    chart.data = data as any

    let yAxis = chart.yAxes.push(new am4charts.ValueAxis());
    yAxis.title.text = "Nanoseconds"
    yAxis.renderer.grid.template.location = 0;
    let xAxis = chart.xAxes.push(new am4charts.ValueAxis());
    xAxis.title.text = "Query ID"

    let series = chart.series.push(new am4charts.LineSeries());
    series.name = "Dijkstra"
    series.data = dijkstra_results
    series.dataFields.valueY = "time"
    series.dataFields.valueX = "query_id"
    series.tooltipText = "{name} {valueX}: \n {valueY}";
    series.strokeWidth = 3

    let seriesBD = chart.series.push(new am4charts.LineSeries());
    seriesBD.name = "Bidirectional Dijkstra"
    seriesBD.data = bd_dijkstra_results
    seriesBD.dataFields.valueY = "time"
    seriesBD.dataFields.valueX = "query_id"
    seriesBD.tooltipText = "{name} {query_id}: \n {time}";
    seriesBD.strokeWidth = 3

    let seriesA = chart.series.push(new am4charts.LineSeries());
    seriesA.name = "AStar"
    seriesA.data = a_star_results
    seriesA.dataFields.valueY = "time"
    seriesA.dataFields.valueX = "query_id"
    seriesA.tooltipText = "{name} {query_id}: \n {time}";
    seriesA.strokeWidth = 3

    chart.cursor = new am4charts.XYCursor();
    chart.cursor.xAxis = xAxis;

    chart.legend = new am4charts.Legend();
  }
}
