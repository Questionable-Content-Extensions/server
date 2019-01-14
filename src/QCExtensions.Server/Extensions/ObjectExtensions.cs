namespace QCExtensions.Server.Extensions
{
	public static class ObjectExtensions
	{
		public static T OrNew<T>(this T @object)
			where T : class, new()
		{
			return @object ?? new T();
		}
	}
}
