import java.nio.file.Files;
import java.nio.file.Paths;

class StreetNetwork {
	static {
		System.loadLibrary("osm2streets_java");
	}

	private static native long create(String osmXmlInput);

	private static native String toGeojsonPlain(long pointer);

	private static native String toLanePolygonsGeojson(long pointer);

	private static native String toLaneMarkingsGeojson(long pointer);

	public static void main(String[] args) throws Exception {
		String osmXmlInput = new String(Files.readAllBytes(Paths.get("../tests/src/aurora_sausage_link/input.osm")));
		long network = create(osmXmlInput);
		System.out.println(toGeojsonPlain(network));
		System.out.println(toLanePolygonsGeojson(network));
		System.out.println(toLaneMarkingsGeojson(network));
	}
}
