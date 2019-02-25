namespace QCExtensions.Domain.Enumerations
{
	public enum ItemType
	{
		Unknown,
		Cast,
		Location,
		Storyline
	}
	
	public static class ItemTypeExtensions
	{
		public static string ToStringRepresentation(this ItemType itemType)
		{
			return itemType.ToString().ToLower();
		}
	}
}
