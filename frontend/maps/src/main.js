// simple approach to calling local backend vs deployed backend
// TODO: support local IPs (192.168.1.*) to allow testing from phone/tablet
const baseUrl =
  window.location.hostname === "localhost"
    ? "http://localhost:8080"
    : "https://api.planting.life";

async function fetchGardens() {
  const gardenLocationsResponse = await fetch(
    `${baseUrl}/gardens?require_precise_location=true`,
    {
      method: "GET",
    },
  );

  if (!gardenLocationsResponse.ok) {
    throw new Error("Failed to fetch gardens");
  }

  return await gardenLocationsResponse.json();
}

async function initMap() {
  const { Map } = await google.maps.importLibrary("maps");
  const { AdvancedMarkerElement, PinElement } =
    await google.maps.importLibrary("marker");

  const map = new Map(document.getElementById("map"), {
    // mapId is setup in GCP here:
    // https://console.cloud.google.com/google/maps-apis/studio/maps
    mapId: "cc83e50772d8beeea536d67c",

    // center the map on Westerville, at a zoom level that gets most the city.
    center: { lat: 40.126252163828525, lng: -82.9321180841486 },
    zoom: 15,

    // disabling these controls removes a lot of clutter, especially on mobile
    cameraControl: false,
    streetViewControl: false,
    rotateControl: false,
    fullscreenControl: false,

    // allows scrolling around w/ one finger, without trying to
    // 'drag to refresh' or otherwise move the viewport around
    gestureHandling: "greedy",
  });

  const gardenLocations = await fetchGardens();

  // draw each marker & circle
  for (const garden of gardenLocations) {
    const gardenCircle = new google.maps.Circle({
      strokeColor: "#00AA00",
      strokeOpacity: 0.8,
      strokeWeight: 1,
      fillColor: "#00AA00",
      fillOpacity: 0.15,
      map,
      center: { lat: garden.latitude, lng: garden.longitude },
      radius: 200, // meters
    });

    const pin = new PinElement({
      background: "#00CC00",
      borderColor: "#007700",
      glyphColor: "#007700",
      scale: 0.7,
    });

    const marker = new AdvancedMarkerElement({
      map,
      position: { lat: garden.latitude, lng: garden.longitude },
      content: pin.element,
    });
  }

  // add the legend to the map
  map.controls[google.maps.ControlPosition.RIGHT_BOTTOM].push(
    document.getElementById("legend"),
  );
}

async function initGoogleMapsApi() {
  const apiKeyResponse = await fetch(`${baseUrl}/maps/api-key`, {
    method: "GET",
  });

  if (!apiKeyResponse.ok) {
    throw new Error("Failed to fetch API key");
  }

  const googleMapsAPIKey = await apiKeyResponse.text();

  // This is a big google-suggested snippet that Prettier formatted "nicely".
  ((g) => {
    var h,
      a,
      k,
      p = "The Google Maps JavaScript API",
      c = "google",
      l = "importLibrary",
      q = "__ib__",
      m = document,
      b = window;
    b = b[c] || (b[c] = {});
    var d = b.maps || (b.maps = {}),
      r = new Set(),
      e = new URLSearchParams(),
      u = () =>
        h ||
        (h = new Promise(async (f, n) => {
          await (a = m.createElement("script"));
          e.set("libraries", [...r] + "");
          for (k in g)
            e.set(
              k.replace(/[A-Z]/g, (t) => "_" + t[0].toLowerCase()),
              g[k],
            );
          e.set("callback", c + ".maps." + q);
          a.src = `https://maps.${c}apis.com/maps/api/js?` + e;
          d[q] = f;
          a.onerror = () => (h = n(Error(p + " could not load.")));
          a.nonce = m.querySelector("script[nonce]")?.nonce || "";
          m.head.append(a);
        }));
    d[l]
      ? console.warn(p + " only loads once. Ignoring:", g)
      : (d[l] = (f, ...n) => r.add(f) && u().then(() => d[l](f, ...n)));
  })({
    key: googleMapsAPIKey,
    v: "weekly",
    // Use the 'v' parameter to indicate the version to use (weekly, beta, alpha, etc.).
    // Add other bootstrap parameters as needed, using camel case.
  });
}

await initGoogleMapsApi();
initMap();
