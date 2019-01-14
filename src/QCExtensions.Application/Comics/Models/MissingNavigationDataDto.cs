namespace QCExtensions.Application.Comics.Models
{
	public class MissingNavigationDataDto
	{
		public NavigationDataDto Cast { get; set; }
		public NavigationDataDto Location { get; set; }
		public NavigationDataDto Storyline { get; set; }
		public NavigationDataDto Title { get; set; }
		public NavigationDataDto Tagline { get; set; }
	}
}
