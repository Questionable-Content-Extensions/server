using System;
using System.Collections.Generic;
using QCExtensions.Domain.Enumerations;
using QCExtensions.Domain.Infrastructure;
using QCExtensions.Domain.ValueObjects;

namespace QCExtensions.Domain.Entities
{
	public class Item
	{
		public Item()
		{
			Occurrences = new HashSet<Occurrence>();
			Images = new HashSet<ItemImage>();
		}
		public int Id { get; set; }
		public string ShortName { get; set; }
		public string Name { get; set; }
		public string Type
		{
			get
			{
				return TypeValue.ToStringRepresentation();
			}
			set
			{
				switch (value)
				{
					case "cast":
						TypeValue = ItemType.Cast;
						break;

					case "location":
						TypeValue = ItemType.Location;
						break;

					case "storyline":
						TypeValue = ItemType.Storyline;
						break;

					default:
						TypeValue = ItemType.Unknown;
						break;
				}
			}
		}
		public ItemType TypeValue { get; set; }
		public HexRgbColor Color { get; set; } = (HexRgbColor)"7F7F7F";
		public ICollection<Occurrence> Occurrences { get; private set; }
		public ICollection<ItemImage> Images { get; private set; }
	}
}
