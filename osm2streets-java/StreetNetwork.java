import java.nio.file.Files;
import java.nio.file.Paths;

class StreetNetwork {
	long pointer;

	StreetNetwork(long pointer) {
		this.pointer = pointer;
	}

	static {
		System.loadLibrary("osm2streets_java");
	}

	private static native StreetNetwork create(String osmXmlInput);

	private native String toGeojsonPlain();

	private native String toLanePolygonsGeojson();

	private native String toLaneMarkingsGeojson();

	public static void main(String[] args) throws Exception {
		String osmXmlInput = new String(Files.readAllBytes(Paths.get("../tests/src/aurora_sausage_link/input.osm")));
		StreetNetwork network = create(osmXmlInput);
		System.out.println(network.toGeojsonPlain());
		System.out.println(network.toLanePolygonsGeojson());
		System.out.println(network.toLaneMarkingsGeojson());
	}
}
