# Fachpraktikum Algorithms on OpenStreetMap data

The backend is contained inside the  `/osm-tasks` folder, the frontend in `/osm-tasks-frontend`. Execution of both applications is described in the corresponding `README.md` files. 

Process to calculate a ship route:

- Start the frontend and backend as described in the `README.md` files
- Press the `Build Graph` button & wait until the graph has been built (or start the backend with the `-b` flag to build the graph on start up)
- Click somewhere on the map. A pop should appear which allows you to select this point as start or destination for the route. 
  - Alternatively you can input the coordinates as latitudes/longitudes (i.e. 51.1657, 10.4515) in the fields.
- Press the `Request Route` button & remember the job id shown in the alert. The job id will be inserted in the job id field and the UI will try to fetch the routing request result after one second. If the result is not ready at this time, you have to use the `Request Calculation Result` button to manually request the result until it is available. 
  - Job ids are increments starting with zero
- Request the route by inserting the job id and pressing the `Request Calculation Result` button

