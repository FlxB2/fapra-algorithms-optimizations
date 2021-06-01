# Fachpraktikum Algorithms on OpenStreetMap data

The backend is contained inside the  `/osm-tasks` folder, the frontend in `/osm-tasks-frontend`. Execution of both applications is described in the corresponding `README.md` files. 

Process to calculate a ship route:

- Start the frontend and backend as described in the `README.md` files
- Press the `Build Graph` button & wait until the graph has been built
- Input the coordinates as latitudes/longitudes, i.e. 51.1657, 10.4515
- Press the `Request Route` button & remember the job id shown in the alert 
  - Job ids are increments starting with zero
- Request the route by inserting the job id and pressing the `Request Calculation Result` button

