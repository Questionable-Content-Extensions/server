namespace QCExtensions.Application.Items.Models
{
	public abstract class ItemDtoBase
	{
		public int Id { get; set; }
		public string ShortName { get; set; }
		public string Name { get; set; }
		public string Type { get; set; }
		public string Color { get; set; }
	}
}
